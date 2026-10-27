// Run the actual imported moves through the production FIGHT/move controller.
// Actor selection uses the completed pack's legal Fairy learners, so this
// regression never grants an unavailable move to manufacture a successful hit.
#[test]
fn northwoods_fairy_moves_execute_through_production_controller() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    let data = shell.runtime.data();
    if !data.moves.contains_key("FAIRY_WIND") {
        return; // Native packs intentionally retain the original type system.
    }
    let mut cases = Vec::new();
    for move_id in [
        "FAIRY_WIND",
        "DISARMING_VOICE",
        "DAZZLING_GLEAM",
        "PLAY_ROUGH",
        "MOONBLAST",
    ] {
        assert_eq!(data.moves[move_id].move_type, "FAIRY");
        let actor = data
            .pokemon
            .iter()
            .find_map(|(id, species)| {
                let is_fairy = species.type1 == "FAIRY" || species.type2 == "FAIRY";
                (is_fairy
                    && data.learnsets.get(id).is_some_and(|learnset| {
                        learnset
                            .iter()
                            .any(|entry| entry.0 <= 60 && entry.1 == move_id)
                    }))
                .then(|| id.clone())
            })
            .expect("each Fairy move needs a legal level-60 Fairy learner");
        let animation_label =
            data.battle_animation_table[usize::from(data.moves[move_id].source_index)].clone();
        cases.push((move_id, actor, animation_label));
    }
    drop(shell);
    for (move_id, actor, animation_label) in cases {
        assert_northwoods_controller_move_hits(move_id, &actor, &animation_label, false);
    }
}

#[test]
fn northwoods_fairy_pack_combat_uses_installed_chart_and_categories() {
    use crate::core::battle::ai::trainer_ai_damage;
    use crate::core::battle::damage::{
        DamageContext, TypeMultiplier, calculate_damage, is_physical_move,
    };
    use crate::core::battle::turn::BattleCombatState;
    use crate::core::models::Pokemon;
    use crate::core::random::Random;

    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    let data = shell.runtime.data();
    let Some(fairy_wind) = data.moves.get("FAIRY_WIND") else {
        return;
    };
    assert!(
        data.type_categories
            .special
            .iter()
            .any(|kind| kind == "FAIRY")
    );
    assert!(!is_physical_move(&data.type_categories, fairy_wind).expect("Fairy Wind category"));
    assert!(
        is_physical_move(&data.type_categories, &data.moves["PLAY_ROUGH"])
            .expect("Play Rough physical override")
    );
    let mut attacker = Pokemon::new_for_tests(
        data.pokemon["MEW"].clone(),
        50,
        Dv::from_non_hp(15, 15, 15, 15),
    );
    let mut defender = attacker.clone();
    attacker.species.type1 = "NORMAL".to_string();
    attacker.species.type2 = "NORMAL".to_string();
    defender.species.type1 = "NORMAL".to_string();
    defender.species.type2 = "NORMAL".to_string();
    let damage = |attacker: &Pokemon, defender: &Pokemon, mov: &crate::core::models::Move| {
        calculate_damage(
            attacker,
            defender,
            mov,
            &data.battle_stat_multipliers,
            &data.type_categories,
            &data.type_effectiveness,
            &data.weather_modifiers,
            DamageContext::default(),
        )
        .expect("actual pack damage calculation")
    };
    let neutral = damage(&attacker, &defender, fairy_wind).damage;
    attacker.species.type2 = "FAIRY".to_string();
    let stab = damage(&attacker, &defender, fairy_wind).damage;
    assert_eq!(stab, neutral * 3 / 2, "secondary Fairy grants actual STAB");
    for (first, second, numerator, denominator) in [
        ("DRAGON", "DARK", 4, 1),
        ("DRAGON", "STEEL", 1, 1),
        ("POISON", "STEEL", 1, 4),
        ("DRAGON", "DRAGON", 2, 1),
    ] {
        defender.species.type1 = first.to_string();
        defender.species.type2 = second.to_string();
        assert_eq!(
            damage(&attacker, &defender, fairy_wind).type_multiplier,
            TypeMultiplier {
                numerator,
                denominator
            },
            "{first}/{second}"
        );
    }
    let mut rng = Random::new(1);
    let state = BattleCombatState::new(defender.clone(), attacker.clone());
    assert_eq!(
        trainer_ai_damage(
            &state,
            fairy_wind,
            &data.battle_stat_multipliers,
            &data.type_categories,
            &data.type_effectiveness,
            &data.weather_modifiers,
            0,
            &mut rng
        )
        .expect("actual AI Fairy damage"),
        damage(&attacker, &defender, fairy_wind).damage
    );

    defender.species.type1 = "NORMAL".to_string();
    defender.species.type2 = "FAIRY".to_string();
    let immune = damage(&attacker, &defender, &data.moves["DRAGONBREATH"]);
    assert_eq!(immune.damage, 0);
    assert_eq!(immune.type_multiplier, TypeMultiplier::zero());
    defender.species.type1 = "STEEL".to_string();
    defender.species.type2 = "STEEL".to_string();
    for move_id in ["BITE", "LICK"] {
        assert_eq!(
            damage(&attacker, &defender, &data.moves[move_id]).type_multiplier,
            TypeMultiplier::one(),
            "modern Steel neutrality to {move_id}"
        );
    }
}

#[test]
fn northwoods_poison_jab_counters_fairy_and_executes_through_controller() {
    use crate::core::battle::damage::TypeMultiplier;
    use crate::core::battle::turn::{
        BattleAction, BattleCombatState, BattleEvent, BattleSide, BattleTurnInput,
        resolve_battle_turn,
    };
    use crate::core::models::{LearnedMove, Pokemon};
    use crate::core::random::Random;

    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    let data = shell.runtime.data();
    let Some(poison_jab) = data.moves.get("POISON_JAB") else {
        return;
    };
    assert!(
        data.learnsets["BARRICANT"]
            .iter()
            .any(|entry| entry.0 <= 60 && entry.1 == "POISON_JAB")
    );
    let mut player = Pokemon::new_for_tests(
        data.pokemon["BARRICANT"].clone(),
        60,
        Dv::from_non_hp(15, 15, 15, 15),
    );
    player.moves.push(LearnedMove {
        name: "POISON_JAB".to_string(),
        current_pp: poison_jab.pp,
        pp_ups: 0,
    });
    let enemy = Pokemon::new_for_tests(
        data.pokemon["CLEFABLE"].clone(),
        60,
        Dv::from_non_hp(15, 15, 15, 15),
    );
    assert_eq!(enemy.species.type1, "FAIRY");
    let mut state = BattleCombatState::new(player, enemy.clone());
    state.enemy_party.push(enemy);
    let mut rng = Random::new(1);
    let outcome = resolve_battle_turn(
        state,
        BattleTurnInput {
            player: BattleAction::Move { slot: 0 },
            enemy: BattleAction::Switch { party_index: 1 },
        },
        &data.moves,
        &data.move_priorities,
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        &mut rng,
    )
    .expect("actual pack Poison Jab counter turn");
    assert_eq!(outcome.state.player.moves[0].current_pp, poison_jab.pp - 1);
    assert!(outcome.events.iter().any(|event| matches!(event,
        BattleEvent::Damage { side: BattleSide::Player, move_name, damage, result, .. }
        if move_name == "POISON_JAB" && *damage > 0 &&
            result.type_multiplier == TypeMultiplier { numerator: 2, denominator: 1 }
    )));
    let animation_label = data.battle_animation_table[usize::from(poison_jab.source_index)].clone();
    drop(shell);
    assert_northwoods_controller_move_hits("POISON_JAB", "BARRICANT", &animation_label, false);
}

#[test]
fn northwoods_fairy_pack_native_donors_compile_with_either_local_label_form() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let shell = route36_battle_shell_for_render_regression();
    if !shell.runtime.data().moves.contains_key("FAIRY_WIND") {
        return;
    }
    let snapshot = shell.shell.snapshot().expect("production battle snapshot");
    for move_id in [
        "FAIRY_WIND",
        "DISARMING_VOICE",
        "DAZZLING_GLEAM",
        "PLAY_ROUGH",
        "MOONBLAST",
        "POISON_JAB",
    ] {
        let definition = visible_move_animation_definition(&snapshot, move_id, 0)
            .expect("installed native donor compiles");
        assert!(definition.1 > 1);
        assert!(
            !definition.4.is_empty() || !definition.5.is_empty(),
            "{move_id}"
        );
    }
    // Compare the actual Gust program with the older equivalent source form,
    // including all event timing rather than only successful parsing.
    let with_colons =
        visible_move_animation_definition(&snapshot, "GUST", 0).expect("native Gust local loop");
    let mut legacy = snapshot;
    for program in std::sync::Arc::make_mut(&mut legacy.presentation)
        .battle_animations
        .values_mut()
    {
        for line in program {
            if line.starts_with('.') && !line.contains(char::is_whitespace) {
                *line = line.trim_end_matches(':').to_string();
            }
        }
    }
    let without_colons =
        visible_move_animation_definition(&legacy, "GUST", 0).expect("legacy Gust local loop");
    assert_eq!(with_colons, without_colons);
    assert_eq!(with_colons.1, 91);
    assert_eq!(with_colons.4.len(), 11);
}

#[test]
fn northwoods_imported_attacks_execute_through_production_controller() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    if !shell.runtime.data().moves.contains_key("POISON_JAB") {
        return;
    }
    let cases = crystal_assets::northwoods_move_animations::IMPORTED_MOVES
        .into_iter()
        .map(|id| (id, northwoods_animation_actor(shell.runtime.data(), id, 60)))
        .collect::<Vec<_>>();
    drop(shell);
    for (id, actor) in cases {
        assert_northwoods_controller_move_hits(
            id,
            &actor,
            &crystal_assets::northwoods_move_animations::animation_label(id),
            true,
        );
    }
}
