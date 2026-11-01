//! Execute the shipped authoring catalog through the authoritative turn engine.
use super::*;

fn authored_move(id: &str) -> Move {
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../planning/minnesota/pokemon/northwoods-moves.json"
    ))
    .expect("Northwoods authoring catalog");
    let value = catalog["moves"]
        .as_array()
        .expect("move array")
        .iter()
        .find(|value| value["id"] == id)
        .expect("authored move");
    serde_json::from_value(value["battle"].clone()).expect("real runtime move shape")
}

fn authored_categories(id: &str) -> TypeCategories {
    let mut categories = type_categories();
    categories.special.push(pokemon_type("FAIRY"));
    let category = if matches!(
        id,
        "WATER_PULSE" | "ENERGY_BALL" | "SIGNAL_BEAM" | "SLIPPER_WALTZ"
    ) {
        crate::battle::damage::MoveCategory::Special
    } else {
        crate::battle::damage::MoveCategory::Physical
    };
    categories.moves.insert(id.to_string(), category);
    categories.moves.insert(
        "TACKLE".to_string(),
        crate::battle::damage::MoveCategory::Physical,
    );
    categories
}

fn authored_priorities() -> MovePriorityTable {
    let mut priorities = move_priorities();
    // The small legacy test fixture omits this supported native effect; real
    // verified packs already carry its ordinary (base) move priority.
    priorities.effect_priorities.insert(
        "SPECIAL_DEFENSE_DOWN_HIT".to_string(),
        priorities.base_priority,
    );
    priorities
}

fn authored_turn(id: &str, seed: u32, hurt: bool) -> BattleTurnOutcome {
    let mov = authored_move(id);
    let mut player = pokemon("MEW", 60, mov.move_type.clone(), id);
    if hurt {
        player.hp = player.max_hp / 2;
    }
    let enemy = pokemon("RATTATA", 80, pokemon_type("NORMAL"), "TACKLE");
    let moves = BTreeMap::from([
        (id.to_string(), mov),
        (
            "TACKLE".to_string(),
            move_data("TACKLE", pokemon_type("NORMAL"), 35, 100),
        ),
    ]);
    let mut rng = Random::new(seed);
    resolve_battle_turn(
        battle_state(player, enemy, rng.seed()),
        BattleTurnInput {
            player: BattleAction::Move { slot: 0 },
            enemy: BattleAction::Switch { party_index: 1 },
        },
        &moves,
        &authored_priorities(),
        &stat_multipliers(),
        &authored_categories(id),
        &type_effectiveness_table(),
        &weather_modifiers(),
        &mut rng,
    )
    .expect("authored move executes a real turn")
}

#[test]
fn northwoods_all_imports_and_signatures_deal_damage_and_spend_pp() {
    for id in [
        "WATER_PULSE",
        "AQUA_JET",
        "ICE_SHARD",
        "ENERGY_BALL",
        "DRAIN_PUNCH",
        "SIGNAL_BEAM",
        "IRON_HEAD",
        "X_SCISSOR",
        "MAGNETIC_RAIL",
        "BOG_SIPHON",
        "TIP_UP_STRIKE",
        "SLIPPER_WALTZ",
        "PIKE_DRILL",
    ] {
        let outcome = authored_turn(id, 1, false);
        assert_eq!(outcome.state.player.moves[0].current_pp, 4, "{id}");
        assert!(outcome.state.enemy.hp < outcome.state.enemy.max_hp, "{id}");
        assert!(
            outcome.events.iter().any(|event| matches!(event,
                BattleEvent::MoveUsed { side: BattleSide::Player, move_name } if move_name == id
            )),
            "{id}"
        );
    }
}

#[test]
fn northwoods_water_ice_and_tipup_priority_beats_a_faster_foe() {
    for id in ["AQUA_JET", "ICE_SHARD", "TIP_UP_STRIKE"] {
        let mov = authored_move(id);
        let player = pokemon("MEW", 10, mov.move_type.clone(), id);
        let enemy = pokemon("RATTATA", 90, pokemon_type("NORMAL"), "TACKLE");
        let moves = BTreeMap::from([
            (id.to_string(), mov),
            (
                "TACKLE".to_string(),
                move_data("TACKLE", pokemon_type("NORMAL"), 35, 100),
            ),
        ]);
        let mut rng = Random::new(1);
        let state = battle_state(player, enemy, rng.seed());
        assert!(state.player.speed < state.enemy.speed);
        let outcome = resolve_battle_turn(
            state,
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: BattleAction::Move { slot: 0 },
            },
            &moves,
            &authored_priorities(),
            &stat_multipliers(),
            &authored_categories(id),
            &type_effectiveness_table(),
            &weather_modifiers(),
            &mut rng,
        )
        .expect("priority move executes");
        assert_eq!(outcome.order.first(), Some(&BattleSide::Player), "{id}");
    }
}

#[test]
fn northwoods_drain_punch_and_bog_siphon_restore_half_actual_damage() {
    for id in ["DRAIN_PUNCH", "BOG_SIPHON"] {
        let outcome = authored_turn(id, 1, true);
        assert!(
            outcome.events.iter().any(|event| matches!(event,
                BattleEvent::HpDrained {
                    side: BattleSide::Player, move_name, damage, amount, hp_before, hp_after, ..
                } if move_name == id && *damage > 0
                    && *amount == (*damage / 2).max(1)
                    && *hp_after == *hp_before + *amount
            )),
            "{id}"
        );
    }
}

#[test]
fn northwoods_secondary_effects_mutate_authoritative_stats_and_confusion() {
    for (id, stat) in [
        ("ENERGY_BALL", Stat::SpecialDefense),
        ("MAGNETIC_RAIL", Stat::Defense),
        ("PIKE_DRILL", Stat::Defense),
    ] {
        assert!(
            (0..128).any(|seed| {
                let outcome = authored_turn(id, seed, false);
                outcome.state.enemy.stat_boosts[&stat] == -1
            }),
            "{id} must actually lower its declared stat"
        );
    }
    for id in ["WATER_PULSE", "SIGNAL_BEAM", "SLIPPER_WALTZ"] {
        assert!(
            (0..128).any(|seed| {
                let outcome = authored_turn(id, seed, false);
                outcome.state.enemy.confusion_turns > 0
                    && outcome.events.iter().any(|event| {
                        matches!(event,
                            BattleEvent::ConfusionApplied {
                                side: BattleSide::Player, target: BattleSide::Enemy, move_name, ..
                            } if move_name == id
                        )
                    })
            }),
            "{id} must actually confuse"
        );
    }
}

#[test]
fn northwoods_iron_head_can_prevent_a_slower_opponents_action() {
    assert!((0..128).any(|seed| {
        let player = pokemon("MEW", 90, pokemon_type("STEEL"), "IRON_HEAD");
        let enemy = pokemon("RATTATA", 10, pokemon_type("NORMAL"), "TACKLE");
        let moves = BTreeMap::from([
            ("IRON_HEAD".to_string(), authored_move("IRON_HEAD")),
            (
                "TACKLE".to_string(),
                move_data("TACKLE", pokemon_type("NORMAL"), 35, 100),
            ),
        ]);
        let mut rng = Random::new(seed);
        let outcome = resolve_battle_turn(
            battle_state(player, enemy, rng.seed()),
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: BattleAction::Move { slot: 0 },
            },
            &moves,
            &authored_priorities(),
            &stat_multipliers(),
            &authored_categories("IRON_HEAD"),
            &type_effectiveness_table(),
            &weather_modifiers(),
            &mut rng,
        )
        .expect("Iron Head turn");
        outcome.state.enemy.hp > 0
            && outcome.state.enemy.hp < outcome.state.enemy.max_hp
            && outcome.events.iter().any(|event| {
                matches!(
                    event,
                    BattleEvent::Flinched {
                        side: BattleSide::Enemy,
                        ..
                    }
                )
            })
            && outcome.state.enemy.moves[0].current_pp == 5
    }));
}

#[test]
fn northwoods_extensions_do_not_alias_native_metronome_bytes() {
    let player = pokemon("CLEFAIRY", 90, pokemon_type("NORMAL"), "METRONOME");
    let enemy = pokemon("RATTATA", 10, pokemon_type("NORMAL"), "TACKLE");
    let mut ember = move_data("EMBER", pokemon_type("FIRE"), 40, 100);
    ember.source_index = 64;
    let moves = BTreeMap::from([
        ("EMBER".to_string(), ember),
        ("WATER_PULSE".to_string(), authored_move("WATER_PULSE")),
        ("PIKE_DRILL".to_string(), authored_move("PIKE_DRILL")),
    ]);
    let state = battle_state(player, enemy, 0);
    let mut rng = ScriptedBattleRandom::new(vec![252, 255, 64]);
    assert_eq!(
        select_metronome_move(&state, BattleSide::Player, &moves, &mut rng),
        Ok((64, "EMBER"))
    );
    assert_eq!(rng.calls, 3);
}
