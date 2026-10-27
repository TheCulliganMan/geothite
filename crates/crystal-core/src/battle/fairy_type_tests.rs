//! Fairy compatibility at the existing data-driven battle boundary.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    ScriptedBattleRandom, apply_conversion2_effect, apply_damage_hit, battle_state,
    hidden_power_type_power, move_data, move_data_with_effect, move_priorities, pokemon,
    stat_multipliers, type_categories, weather_modifiers,
};
use crate::battle::ai::trainer_ai_damage;
use crate::battle::damage::{
    DamageContext, MoveCategory, TypeCategories, TypeEffectivenessEntry, TypeEffectivenessTable,
    TypeMultiplier, calculate_damage, is_physical_move,
};
use crate::battle::turn::{
    BattleAction, BattleEvent, BattleSide, BattleTurnInput, resolve_battle_turn,
};
use crate::models::{Dv, Pokemon, pokemon_type};

fn fairy_categories() -> TypeCategories {
    let mut categories = type_categories();
    categories.special.push(pokemon_type("FAIRY"));
    categories
}

fn fairy_chart() -> TypeEffectivenessTable {
    let matchups = [
        ("FAIRY", "DRAGON", 2, 1),
        ("FAIRY", "DARK", 2, 1),
        ("FAIRY", "FIGHTING", 2, 1),
        ("FAIRY", "STEEL", 1, 2),
        ("FAIRY", "POISON", 1, 2),
        ("FAIRY", "FIRE", 1, 2),
        ("DRAGON", "FAIRY", 0, 1),
        ("DRAGON", "STEEL", 1, 2),
    ]
    .into_iter()
    .map(
        |(attacker, defender, numerator, denominator)| TypeEffectivenessEntry {
            attacker: pokemon_type(attacker),
            defender: pokemon_type(defender),
            multiplier: TypeMultiplier {
                numerator,
                denominator,
            },
        },
    )
    .collect();
    TypeEffectivenessTable {
        matchups,
        foresight_matchups: Vec::new(),
    }
}

fn fairy_damage(attacker: &Pokemon, defender: &Pokemon) -> crate::battle::damage::DamageResult {
    calculate_damage(
        attacker,
        defender,
        &move_data("FAIRY_WIND", pokemon_type("FAIRY"), 40, 100),
        &stat_multipliers(),
        &fairy_categories(),
        &fairy_chart(),
        &weather_modifiers(),
        DamageContext::default(),
    )
    .expect("declared Fairy damage resolves")
}

#[test]
fn fairy_damage_applies_stab_and_distinct_dual_types() {
    let mut attacker = pokemon("ATTACKER", 90, pokemon_type("NORMAL"), "FAIRY_WIND");
    let mut defender = pokemon("DEFENDER", 40, pokemon_type("NORMAL"), "TACKLE");
    let neutral = fairy_damage(&attacker, &defender).damage;
    attacker.species.type2 = pokemon_type("FAIRY");
    let stab = fairy_damage(&attacker, &defender).damage;
    assert_eq!(stab, neutral * 3 / 2);
    for (first, second, multiplier, damage) in [
        (
            "DRAGON",
            "DARK",
            TypeMultiplier {
                numerator: 4,
                denominator: 1,
            },
            stab * 4,
        ),
        ("DRAGON", "STEEL", TypeMultiplier::one(), stab),
        (
            "POISON",
            "STEEL",
            TypeMultiplier {
                numerator: 1,
                denominator: 4,
            },
            stab / 2 / 2,
        ),
        (
            "DRAGON",
            "DRAGON",
            TypeMultiplier {
                numerator: 2,
                denominator: 1,
            },
            stab * 2,
        ),
    ] {
        defender.species.type1 = pokemon_type(first);
        defender.species.type2 = pokemon_type(second);
        let result = fairy_damage(&attacker, &defender);
        assert_eq!(result.type_multiplier, multiplier, "{first}/{second}");
        assert_eq!(result.damage, damage, "{first}/{second}");
    }
}

#[test]
fn fairy_turn_spends_both_pp_and_blocks_dragon_damage() {
    let player = pokemon("FAIRY", 90, pokemon_type("FAIRY"), "FAIRY_WIND");
    let mut enemy = pokemon("DRAGON", 40, pokemon_type("DRAGON"), "DRAGONBREATH");
    enemy.species.type2 = pokemon_type("STEEL");
    let player_hp = player.hp;
    let moves = BTreeMap::from([
        (
            "FAIRY_WIND".to_string(),
            move_data("FAIRY_WIND", pokemon_type("FAIRY"), 40, 100),
        ),
        (
            "DRAGONBREATH".to_string(),
            move_data("DRAGONBREATH", pokemon_type("DRAGON"), 60, 100),
        ),
    ]);
    let mut rng = ScriptedBattleRandom::new(vec![241; 32]);
    let outcome = resolve_battle_turn(
        battle_state(player, enemy, 0),
        BattleTurnInput {
            player: BattleAction::Move { slot: 0 },
            enemy: BattleAction::Move { slot: 0 },
        },
        &moves,
        &move_priorities(),
        &stat_multipliers(),
        &fairy_categories(),
        &fairy_chart(),
        &weather_modifiers(),
        &mut rng,
    )
    .expect("Fairy turn resolves");
    assert_eq!(outcome.state.player.moves[0].current_pp, 4);
    assert_eq!(outcome.state.enemy.moves[0].current_pp, 4);
    assert_eq!(outcome.state.player.hp, player_hp);
    assert!(outcome.state.enemy.hp < outcome.state.enemy.max_hp);
    assert!(outcome.events.iter().any(|event| matches!(event,
        BattleEvent::NoEffect { side: BattleSide::Enemy, move_name } if move_name == "DRAGONBREATH"
    )));
}

#[test]
fn fairy_ai_damage_respects_immunity_and_move_category_override() {
    let player = pokemon("FAIRY", 90, pokemon_type("FAIRY"), "FAIRY_WIND");
    let enemy = pokemon("DRAGON", 40, pokemon_type("DRAGON"), "DRAGONBREATH");
    let state = battle_state(player, enemy, 0);
    let dragon_move = move_data("DRAGONBREATH", pokemon_type("DRAGON"), 60, 100);
    let mut rng = ScriptedBattleRandom::new(Vec::new());
    assert_eq!(
        trainer_ai_damage(
            &state,
            &dragon_move,
            &stat_multipliers(),
            &fairy_categories(),
            &fairy_chart(),
            &weather_modifiers(),
            0,
            &mut rng,
        )
        .expect("AI calculates Fairy immunity"),
        0
    );
    assert_eq!(rng.calls, 0);

    let play_rough = move_data("PLAY_ROUGH", pokemon_type("FAIRY"), 90, 90);
    let mut categories = fairy_categories();
    assert!(!is_physical_move(&categories, &play_rough).expect("default special split"));
    categories
        .moves
        .insert("PLAY_ROUGH".to_string(), MoveCategory::Physical);
    assert!(is_physical_move(&categories, &play_rough).expect("modern physical override"));
}

#[test]
fn fairy_mod_preserves_hidden_power_native_sixteen_type_pool() {
    let mut user = pokemon("UNOWN", 40, pokemon_type("PSYCHIC_TYPE"), "HIDDEN_POWER");
    let mut observed = BTreeSet::new();
    for attack in 0..16 {
        for defense in 0..16 {
            user.dvs = Dv::from_non_hp(attack, defense, 10, 10);
            observed.insert(hidden_power_type_power(&user).0);
        }
    }
    let expected = [
        "FIGHTING",
        "FLYING",
        "POISON",
        "GROUND",
        "ROCK",
        "BUG",
        "GHOST",
        "STEEL",
        "FIRE",
        "WATER",
        "GRASS",
        "ELECTRIC",
        "PSYCHIC_TYPE",
        "ICE",
        "DRAGON",
        "DARK",
    ]
    .into_iter()
    .map(pokemon_type)
    .collect();
    assert_eq!(observed, expected);
}

#[test]
fn fairy_mod_conversion2_rejects_new_numeric_slot_despite_chart_immunity() {
    let player = pokemon("PORYGON", 90, pokemon_type("NORMAL"), "CONVERSION2");
    let enemy = pokemon("DRAGON", 40, pokemon_type("DRAGON"), "DRAGONBREATH");
    let mut state = battle_state(player, enemy, 0);
    state.enemy_last_counter_move = Some("DRAGONBREATH".to_string());
    let moves = BTreeMap::from([(
        "DRAGONBREATH".to_string(),
        move_data("DRAGONBREATH", pokemon_type("DRAGON"), 60, 100),
    )]);
    let mut rng = ScriptedBattleRandom::new(vec![28, 9]);
    let mut events = Vec::new();
    apply_conversion2_effect(
        &mut state,
        BattleSide::Player,
        "CONVERSION2",
        &moves,
        &fairy_chart(),
        &mut rng,
        &mut events,
    )
    .expect("native Conversion2 resolves");
    assert_eq!(
        state.player_type_override.expect("converted type").type1,
        "STEEL"
    );
    assert_eq!(rng.calls, 2);
}

#[test]
fn fairy_mod_link_present_accepts_both_fairy_type_registers() {
    let player = pokemon("DELIBIRD", 90, pokemon_type("FAIRY"), "PRESENT");
    let mut enemy = pokemon("CLEFABLE", 40, pokemon_type("FAIRY"), "SPLASH");
    enemy.hp = 200;
    enemy.max_hp = 200;
    let mut state = crate::battle::turn::BattleCombatState::new(player, enemy);
    state.link_battle = true;
    state.link_colosseum = true;
    let present = move_data_with_effect("PRESENT", pokemon_type("NORMAL"), 1, 100, "PRESENT");
    let mut rng = ScriptedBattleRandom::new(vec![255, 0, 255]);
    let mut events = Vec::new();
    apply_damage_hit(
        &mut state,
        BattleSide::Player,
        "PRESENT",
        &present,
        1,
        false,
        false,
        &stat_multipliers(),
        &fairy_categories(),
        &fairy_chart(),
        &weather_modifiers(),
        &BTreeMap::new(),
        &mut rng,
        &mut events,
    )
    .expect("mod Fairy link Present resolves");
    assert!(events.iter().any(|event| matches!(event,
        BattleEvent::Damage { damage, .. } if *damage > 0
    )));
    assert!(state.enemy.hp < 200);
}

#[test]
fn fairy_moonblast_authored_secondary_has_exact_thirty_percent_boundary() {
    use crate::battle::turn::battle_move_effect_is_supported;
    use crate::models::{Move, Stat};

    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../planning/minnesota/pokemon/fairy-moves.json"
    ))
    .expect("authored Fairy move catalog");
    let entry = catalog["moves"]
        .as_array()
        .expect("authored move array")
        .iter()
        .find(|entry| entry["id"] == "MOONBLAST")
        .expect("authored Moonblast");
    let moonblast: Move =
        serde_json::from_value(entry["battle"].clone()).expect("real runtime Moonblast definition");
    assert!(battle_move_effect_is_supported(&moonblast.effect));
    assert_eq!(moonblast.effect_chance, 30);
    assert_eq!(
        (moonblast.stat, moonblast.amount),
        (Some(Stat::SpecialAttack), Some(-1))
    );
    for (roll, expected_stage) in [(75, -1), (76, 0)] {
        let player = pokemon("CLEFABLE", 90, pokemon_type("FAIRY"), "MOONBLAST");
        let mut enemy = pokemon("MEW", 40, pokemon_type("NORMAL"), "SPLASH");
        enemy.hp = 200;
        enemy.max_hp = 200;
        let mut state = crate::battle::turn::BattleCombatState::new(player, enemy);
        let mut events = Vec::new();
        // Shared hit preparation samples critical and variation before the
        // pre-ApplyDamage secondary-effect command.
        let mut rng = ScriptedBattleRandom::new(vec![255, 255, roll]);
        apply_damage_hit(
            &mut state,
            BattleSide::Player,
            "MOONBLAST",
            &moonblast,
            1,
            false,
            false,
            &stat_multipliers(),
            &fairy_categories(),
            &fairy_chart(),
            &weather_modifiers(),
            &BTreeMap::new(),
            &mut rng,
            &mut events,
        )
        .expect("authored Moonblast hit resolves");
        assert!(state.enemy.hp < 200);
        assert_eq!(
            state.enemy.stat_boosts[&Stat::SpecialAttack],
            expected_stage,
            "secondary roll {roll}"
        );
        assert_eq!(state.enemy.stat_boosts[&Stat::SpecialDefense], 0);
        assert_eq!(rng.calls, 3);
    }
}
