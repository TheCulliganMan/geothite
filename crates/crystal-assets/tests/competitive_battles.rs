use crystal_assets::{
    GameDataSet, TrainerBattleProfile, build_competitive_battle_modpack,
    read_verified_compiled_game_pack,
};
use crystal_core::{
    battle::turn::{BattleAction, BattleCombatState, BattleDisableState, BattleTurnInput},
    models::{Dv, LearnedMove, Pokemon},
    random::BattleRandomSource,
};
use std::{path::PathBuf, sync::OnceLock, time::Instant};

fn base_path() -> PathBuf {
    std::env::var_os("CRYSTAL_MODPACK_TEST_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/core-modular.browser.crystalpack")
        })
        .canonicalize()
        .unwrap()
}

fn data() -> &'static GameDataSet {
    static DATA: OnceLock<GameDataSet> = OnceLock::new();
    DATA.get_or_init(|| {
        let base = read_verified_compiled_game_pack(base_path()).unwrap();
        build_competitive_battle_modpack(&base)
            .unwrap()
            .data()
            .clone()
    })
}

struct Random(u32);
impl BattleRandomSource for Random {
    fn battle_random_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 24) as u8
    }
}

fn pokemon(species: &str, moves: &[&str]) -> Pokemon {
    let data = data();
    let mut p = data
        .create_pokemon(species, 50, Dv::from_non_hp(15, 15, 15, 15))
        .unwrap();
    p.moves = moves
        .iter()
        .map(|name| LearnedMove {
            name: name.to_string(),
            current_pp: data.moves[*name].pp,
            pp_ups: 0,
        })
        .collect();
    p
}

fn select(state: &BattleCombatState) -> usize {
    data()
        .select_trainer_enemy_move_slot(state, 0, &mut Random(42))
        .unwrap()
}

const PROFILE_CLASSES: [&str; 6] = [
    "CHAMPION",
    "YOUNGSTER",
    "PSYCHIC_T",
    "BUG_CATCHER",
    "FISHER",
    "BIRD_KEEPER",
];

fn select_class(state: &BattleCombatState, class: &str) -> usize {
    data()
        .select_trainer_enemy_move_for_class(state, class, 0, &mut Random(42))
        .unwrap()
        .slot
}

#[test]
fn every_trainer_profile_takes_a_priority_win_with_all_authored_flags_off() {
    let mut state = BattleCombatState::new(
        pokemon("ALAKAZAM", &["PSYCHIC_M"]),
        pokemon(
            "TYPHLOSION",
            &["FLAMETHROWER", "QUICK_ATTACK", "SMOKESCREEN"],
        ),
    );
    state.player.hp = 1;
    state.enemy.hp = 1;
    state.player_used_moves = vec!["PSYCHIC_M".into()];
    for (class, profile) in PROFILE_CLASSES.into_iter().zip(TrainerBattleProfile::ALL) {
        assert_eq!(TrainerBattleProfile::for_class(class), profile);
        assert_eq!(
            select_class(&state, class),
            1,
            "{class}: take the guaranteed priority knockout"
        );
    }
    assert_eq!(
        TrainerBattleProfile::for_class("CUSTOM_TRAINER"),
        TrainerBattleProfile::Balanced
    );
    assert_eq!(
        TrainerBattleProfile::for_class("RIVAL2"),
        TrainerBattleProfile::Balanced
    );
}

#[test]
fn competitive_ai_mixes_counter_and_mirror_coat_against_physical_and_special_threats() {
    struct Byte {
        value: u8,
        calls: u8,
    }
    impl BattleRandomSource for Byte {
        fn battle_random_byte(&mut self) -> u8 {
            self.calls += 1;
            self.value
        }
    }
    let mut state = BattleCombatState::new(
        pokemon("MEW", &["PSYCHIC_M", "EARTHQUAKE", "THUNDERBOLT", "SURF"]),
        pokemon("WOBBUFFET", &["COUNTER", "MIRROR_COAT"]),
    );
    state.player_used_moves = state.player.moves.iter().map(|m| m.name.clone()).collect();
    let before = state.clone();
    let choices = [0, 255].map(|value| {
        let mut rng = Byte { value, calls: 0 };
        let selected = data()
            .select_trainer_enemy_move_for_class(&state, "CHAMPION", 0, &mut rng)
            .unwrap();
        assert_eq!(
            rng.calls, 1,
            "only the real policy draw may consume live RNG"
        );
        selected.slot
    });
    assert_ne!(
        choices[0], choices[1],
        "each pure counter can be exploited: {choices:?}"
    );
    assert_eq!(state, before, "hypothetical turns must not mutate combat");
    for (slot, player_slot) in [(0, 1), (1, 0)] {
        let outcome = data()
            .resolve_battle_turn_with_items(
                state.clone(),
                BattleTurnInput {
                    player: BattleAction::Move { slot: player_slot },
                    enemy: BattleAction::Move { slot },
                },
                &mut Random(47),
            )
            .unwrap();
        assert!(
            outcome.state.player.hp < state.player.hp,
            "selected counter must actually retaliate"
        );
        assert_eq!(
            outcome.state.enemy.moves[slot].current_pp,
            state.enemy.moves[slot].current_pp - 1
        );
    }
}

#[test]
fn trainer_profiles_can_choose_distinct_plans_in_close_recovery_positions() {
    let moves = ["RAZOR_LEAF", "SYNTHESIS", "REFLECT", "BODY_SLAM"];
    let mut state =
        BattleCombatState::new(pokemon("MEGANIUM", &moves), pokemon("MEGANIUM", &moves));
    state.player_used_moves = moves.iter().map(|m| m.to_string()).collect();
    state.enemy.hp = state.enemy.max_hp * 45 / 100;
    state.player.hp = state.player.max_hp / 2;
    assert_eq!(
        select_class(&state, "YOUNGSTER"),
        3,
        "offensive style presses with Body Slam"
    );
    assert_eq!(
        select_class(&state, "FISHER"),
        1,
        "endurance style sustains with Synthesis"
    );
}

#[test]
fn every_trainer_profile_switches_out_of_a_losing_matchup_and_respects_trapping() {
    let mut d = data().clone();
    let id = d.trainers.trainers.keys().next().unwrap().clone();
    let mut state = BattleCombatState::new(
        pokemon("RAICHU", &["THUNDERBOLT"]),
        pokemon("GYARADOS", &["SURF", "BITE"]),
    );
    state.player_used_moves = vec!["THUNDERBOLT".into()];
    state
        .enemy_party
        .push(pokemon("DONPHAN", &["EARTHQUAKE", "BODY_SLAM"]));
    for class in PROFILE_CLASSES {
        let trainer = d.trainers.trainers.get_mut(&id).unwrap();
        trainer.trainer_class = class.into();
        trainer.items.clear();
        let slot = select_class(&state, class);
        let action = d
            .select_trainer_post_order_action(
                &state,
                &id,
                "BATTLETYPE_NORMAL",
                0,
                &mut Default::default(),
                slot,
                &mut Random(1),
            )
            .unwrap();
        assert!(
            matches!(action, BattleAction::TrainerSwitch { party_index: 1, .. }),
            "{class}: {action:?}"
        );
    }
    // Mean Look's authoritative escape trap blocks even the switching profile.
    state.enemy_escape_trap = Some(crystal_core::battle::turn::BattleEscapeTrapState {
        source: crystal_core::battle::turn::BattleSide::Player,
        move_name: "MEAN_LOOK".into(),
    });
    let action = d
        .select_trainer_post_order_action(
            &state,
            &id,
            "BATTLETYPE_NORMAL",
            0,
            &mut Default::default(),
            0,
            &mut Random(1),
        )
        .unwrap();
    assert!(matches!(action, BattleAction::Move { .. }), "{action:?}");
}

#[test]
fn a_weak_revealed_move_does_not_erase_public_threats_or_grant_tower_items() {
    let mut state = BattleCombatState::new(
        pokemon("RAICHU", &["GROWL", "THUNDERBOLT"]),
        pokemon("GYARADOS", &["SURF", "BITE"]),
    );
    state.player_used_moves = vec!["GROWL".into()];
    state
        .enemy_party
        .push(pokemon("DONPHAN", &["EARTHQUAKE", "BODY_SLAM"]));
    let slot = select_class(&state, "BIRD_KEEPER");
    let mut used = Default::default();
    let action = data()
        .select_trainer_post_order_action_for_class(
            &state,
            "GENERATED_TOWER_OPPONENT",
            "BIRD_KEEPER",
            "BATTLETYPE_BATTLE_TOWER",
            0,
            &mut used,
            slot,
            &mut Random(1),
        )
        .unwrap();
    assert!(
        matches!(action, BattleAction::TrainerSwitch { party_index: 1, .. }),
        "do not assume Growl is Raichu's only threat: {action:?}"
    );
    assert!(
        used.is_empty(),
        "generated Tower opponents must not receive consumables"
    );
    // The prior comes from public learnsets, independently of hidden slots.
    state.player.moves = pokemon("RAICHU", &["TACKLE", "TAIL_WHIP"]).moves;
    assert_eq!(select_class(&state, "BIRD_KEEPER"), slot);
    let again = data()
        .select_trainer_post_order_action_for_class(
            &state,
            "GENERATED_TOWER_OPPONENT",
            "BIRD_KEEPER",
            "BATTLETYPE_BATTLE_TOWER",
            0,
            &mut used,
            slot,
            &mut Random(1),
        )
        .unwrap();
    assert_eq!(again, action);
}

#[test]
fn competitive_items_ignore_legacy_probability_gates_but_preserve_inventory() {
    let mut d = data().clone();
    let id = d.trainers.trainers.keys().next().unwrap().clone();
    let trainer = d.trainers.trainers.get_mut(&id).unwrap();
    trainer.trainer_class = "YOUNGSTER".into();
    trainer.items = vec![Some("POTION".into()), Some("FULL_RESTORE".into())];
    let mut state = BattleCombatState::new(
        pokemon("SHUCKLE", &["CONSTRICT"]),
        pokemon("ALAKAZAM", &["TACKLE"]),
    );
    state.player_used_moves = vec!["CONSTRICT".into()];
    state.enemy.hp = 1;
    // A stronger reserve must not stop the trainer saving its current battler.
    let mut reserve = pokemon("MAGIKARP", &["SPLASH"]);
    reserve.level = 51;
    state.enemy_party.push(reserve);
    let mut used = Default::default();
    let action = d
        .select_trainer_post_order_action(
            &state,
            &id,
            "BATTLETYPE_NORMAL",
            0,
            &mut used,
            0,
            &mut Random(0),
        )
        .unwrap();
    assert!(
        matches!(&action, BattleAction::TrainerItem { item_id, .. } if item_id == "FULL_RESTORE"),
        "{action:?}"
    );
    assert_eq!(
        used,
        std::collections::BTreeSet::from([format!("{id}:FULL_RESTORE:1")])
    );
    let outcome = d
        .resolve_battle_turn_with_items(
            state,
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: action,
            },
            &mut Random(4),
        )
        .unwrap();
    assert!(
        outcome.state.enemy.hp > 1,
        "the selected item must heal through the production engine"
    );
}

#[test]
fn competitive_pack_roundtrips_and_preserves_all_unrelated_content_and_audio() {
    let base = read_verified_compiled_game_pack(base_path()).unwrap();
    let pack = build_competitive_battle_modpack(&base).unwrap();
    assert_eq!(
        base.identity().unwrap() == pack.identity().unwrap(),
        base.data().competitive_battles
    );
    assert_eq!(pack.data().type_categories.moves.len(), 251);
    assert!(pack.data().competitive_battles);
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    // Includes every PCM hash, frame count, and loop range in the local pack.
    assert_eq!(
        pack.audio_manifest().unwrap(),
        base.audio_manifest().unwrap()
    );
    let mut unchanged = pack.data().clone();
    unchanged.competitive_battles = base.data().competitive_battles;
    unchanged.type_categories = base.data().type_categories.clone();
    assert_eq!(&unchanged, base.data());
    assert_eq!(
        build_competitive_battle_modpack(&pack)
            .unwrap()
            .identity()
            .unwrap(),
        pack.identity().unwrap()
    );
    let output = std::env::temp_dir().join(format!(
        "competitive-battles-{}.crystalpack",
        std::process::id()
    ));
    pack.write_preserving_storage(&output).unwrap();
    let loaded = read_verified_compiled_game_pack(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
    assert_eq!(loaded.data(), pack.data());
}

#[test]
fn competitive_ai_uses_split_stats_and_avoids_immunity_with_all_trainer_flags_off() {
    let mut state = BattleCombatState::new(
        pokemon("GENGAR", &["LICK"]),
        pokemon(
            "ALAKAZAM",
            &["TACKLE", "PSYCHIC_M", "SHADOW_BALL", "RECOVER"],
        ),
    );
    state.player_used_moves = vec!["LICK".into()];
    let started = Instant::now();
    assert_eq!(select(&state), 1);
    eprintln!("competitive decision elapsed: {:?}", started.elapsed());
    let original = state.clone();
    let slot = select(&state);
    assert_eq!(
        state, original,
        "planning must not mutate authoritative combat"
    );
    let pp = state.enemy.moves[slot].current_pp;
    let outcome = data()
        .resolve_battle_turn_with_items(
            state,
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: BattleAction::Move { slot },
            },
            &mut Random(47),
        )
        .unwrap();
    assert_eq!(outcome.state.enemy.moves[slot].current_pp, pp - 1);
    assert!(outcome.state.player.hp < original.player.hp);
}

#[test]
fn competitive_ai_respects_pp_disable_and_does_not_read_hidden_moves_or_bench() {
    let mut state = BattleCombatState::new(
        pokemon("BLASTOISE", &["SURF"]),
        pokemon(
            "ALAKAZAM",
            &["THUNDERPUNCH", "PSYCHIC_M", "SHADOW_BALL", "RECOVER"],
        ),
    );
    state.player_used_moves = vec!["SURF".into()];
    let expected = select(&state);
    state.player.moves = pokemon(
        "BLASTOISE",
        &["EXPLOSION", "HYPER_BEAM", "TOXIC", "PROTECT"],
    )
    .moves;
    state.player_party = vec![state.player.clone(), pokemon("MEWTWO", &["PSYCHIC_M"])];
    state.player.item = Some("LEFTOVERS".into());
    assert_eq!(select(&state), expected);
    state.enemy_disable = Some(BattleDisableState {
        move_name: state.enemy.moves[expected].name.clone(),
        turns_remaining: 3,
    });
    assert_ne!(select(&state), expected);
    state.enemy_disable = None;
    state.enemy.moves[expected].current_pp = 0;
    assert_ne!(select(&state), expected);
}

#[test]
fn competitive_ai_prefers_a_reliable_two_hit_ko_over_fishing_for_paralysis() {
    let p = pokemon(
        "ALAKAZAM",
        &["PSYCHIC_M", "SHADOW_BALL", "THUNDERPUNCH", "RECOVER"],
    );
    let mut state = BattleCombatState::new(p.clone(), p);
    state.player_used_moves = state.player.moves.iter().map(|m| m.name.clone()).collect();
    if std::env::var_os("CRYSTAL_BATTLE_AI_TRACE").is_some() {
        for learned in &state.enemy.moves {
            eprintln!(
                "{:?}",
                crystal_core::battle::turn::preview_battle_move(
                    &state,
                    crystal_core::battle::turn::BattleSide::Enemy,
                    &data().moves[&learned.name],
                    &data().items,
                    &data().move_priorities,
                    &data().battle_stat_multipliers,
                    &data().type_categories,
                    &data().type_effectiveness,
                    &data().weather_modifiers
                )
                .unwrap()
            );
        }
    }
    assert_eq!(
        select(&state),
        1,
        "Shadow Ball wins the damage race; Thunderpunch needs luck"
    );
}

#[test]
fn competitive_ai_recovers_and_switches_out_of_losing_matchups() {
    let mut recovery = BattleCombatState::new(
        pokemon("SHUCKLE", &["CONSTRICT"]),
        pokemon("ALAKAZAM", &["TACKLE", "RECOVER"]),
    );
    recovery.player_used_moves = vec!["CONSTRICT".into()];
    recovery.enemy.hp = recovery.enemy.max_hp / 4;
    assert_eq!(select(&recovery), 1);

    let mut state = BattleCombatState::new(
        pokemon("RAICHU", &["THUNDERBOLT"]),
        pokemon("GYARADOS", &["SURF", "BITE"]),
    );
    state.player_used_moves = vec!["THUNDERBOLT".into()];
    state
        .enemy_party
        .push(pokemon("DONPHAN", &["EARTHQUAKE", "BODY_SLAM"]));
    let slot = select(&state);
    let trainer_id = data()
        .trainers
        .trainers
        .iter()
        .find(|(_, t)| t.trainer_class == "CHAMPION")
        .unwrap()
        .0;
    let action = data()
        .select_trainer_post_order_action(
            &state,
            trainer_id,
            "BATTLETYPE_NORMAL",
            0,
            &mut Default::default(),
            slot,
            &mut Random(1),
        )
        .unwrap();
    assert!(
        matches!(action, BattleAction::TrainerSwitch { party_index: 1, .. }),
        "{action:?}"
    );
    let out = data()
        .resolve_battle_turn_with_items(
            state,
            BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: action,
            },
            &mut Random(4),
        )
        .unwrap();
    assert_eq!(out.state.enemy.species.id, "DONPHAN");
    assert_eq!(
        out.state.enemy.hp, out.state.enemy.max_hp,
        "switch must actually absorb the electric move"
    );
}

#[test]
fn competitive_ai_uses_owned_healing_items_once_without_inventing_inventory() {
    let mut d = data().clone();
    let trainer_id = d.trainers.trainers.keys().next().unwrap().clone();
    d.trainers.trainers.get_mut(&trainer_id).unwrap().items = vec![Some("FULL_RESTORE".into())];
    let mut state = BattleCombatState::new(
        pokemon("SHUCKLE", &["CONSTRICT"]),
        pokemon("ALAKAZAM", &["TACKLE"]),
    );
    state.player_used_moves = vec!["CONSTRICT".into()];
    state.enemy.hp = state.enemy.max_hp / 8;
    state.enemy_party[0] = state.enemy.clone();
    let mut used = Default::default();
    let action = d
        .select_trainer_post_order_action(
            &state,
            &trainer_id,
            "BATTLETYPE_NORMAL",
            0,
            &mut used,
            0,
            &mut Random(1),
        )
        .unwrap();
    assert!(
        matches!(action, BattleAction::TrainerItem { ref item_id, .. } if item_id == "FULL_RESTORE"),
        "{action:?}"
    );
    let action = d
        .select_trainer_post_order_action(
            &state,
            &trainer_id,
            "BATTLETYPE_NORMAL",
            0,
            &mut used,
            0,
            &mut Random(1),
        )
        .unwrap();
    assert!(
        matches!(action, BattleAction::Move { .. }),
        "item must be exhausted: {action:?}"
    );
}

#[test]
fn competitive_move_previews_match_real_damage_and_resolve_hidden_power() {
    use crystal_core::battle::{
        damage::MoveCategory,
        turn::{BattleEvent, BattleSide, preview_battle_move},
    };
    let d = data();
    let mut state = BattleCombatState::new(
        pokemon("ALAKAZAM", &["SHADOW_BALL", "THUNDERPUNCH", "HIDDEN_POWER"]),
        pokemon("BLASTOISE", &["WITHDRAW"]),
    );
    let preview = |state: &BattleCombatState, id: &str| {
        preview_battle_move(
            state,
            BattleSide::Player,
            &d.moves[id],
            &d.items,
            &d.move_priorities,
            &d.battle_stat_multipliers,
            &d.type_categories,
            &d.type_effectiveness,
            &d.weather_modifiers,
        )
        .unwrap()
    };
    assert_eq!(
        preview(&state, "SHADOW_BALL").category,
        MoveCategory::Special
    );
    assert_eq!(
        preview(&state, "THUNDERPUNCH").category,
        MoveCategory::Physical
    );
    let hidden = preview(&state, "HIDDEN_POWER");
    assert_eq!(hidden.move_type, "DARK");
    assert_eq!(hidden.power, Some(70));
    assert!(preview(&state, "MAGNITUDE").damage_on_hit.is_none());
    assert!(preview(&state, "RECOVER").damage_on_hit.is_none());
    let original = state.clone();
    let [min, max] = preview(&state, "SHADOW_BALL").damage_on_hit.unwrap();
    let mut seen = 0;
    for seed in 1..=32 {
        let outcome = d
            .resolve_battle_turn_with_items(
                state.clone(),
                BattleTurnInput {
                    player: BattleAction::Move { slot: 0 },
                    enemy: BattleAction::Move { slot: 0 },
                },
                &mut Random(seed),
            )
            .unwrap();
        for event in outcome.events {
            if let BattleEvent::Damage {
                side: BattleSide::Player,
                critical: false,
                result,
                ..
            } = event
            {
                assert!(
                    (min..=max).contains(&result.damage),
                    "{min} <= {} <= {max}",
                    result.damage
                );
                seen += 1;
            }
        }
    }
    assert!(seen >= 24);
    assert_eq!(state, original);
    state.enemy_light_screen_turns = 5;
    assert!(preview(&state, "SHADOW_BALL").damage_on_hit.unwrap()[1] < max);
}

#[test]
fn competitive_replacement_selects_a_counter_without_reordering_party_or_spending_a_turn() {
    use crystal_core::{
        battle::start::TrainerBattleStart,
        models::Stat,
        state::{BattleMemory, GameState},
    };
    let mut first = pokemon("RATTATA", &["TACKLE"]);
    first.hp = 0;
    let party = vec![
        first.clone(),
        pokemon("GYARADOS", &["SURF", "BITE"]),
        pokemon("DONPHAN", &["EARTHQUAKE", "BODY_SLAM"]),
    ];
    let trainer = data()
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "CHAMPION")
        .unwrap();
    let start = TrainerBattleStart {
        battle_type: "BATTLETYPE_NORMAL".into(),
        trainer_class: trainer.trainer_class.clone(),
        trainer_id: trainer.trainer_id.clone(),
        trainer_name: trainer.name.clone(),
        event_flag: "EVENT_BEAT_CHAMPION_LANCE".into(),
        seen_text: String::new(),
        win_text: String::new(),
        loss_text: String::new(),
        callback: String::new(),
        source_script: String::new(),
        enemy_pokemon: first.clone(),
        enemy_party: party.clone(),
        reward: trainer.base_reward,
        encounter_music: trainer.encounter_music.clone(),
        ai_move_flags: 0,
        ai_item_switch_flags: 0,
        ai_layers: Vec::new(),
    };
    let mut state = GameState::default();
    state.battle = BattleMemory::from(&start);
    state.battle_active_enemy_party_index = Some(0);
    let mut combat = BattleCombatState::new(pokemon("RAICHU", &["THUNDERBOLT"]), first);
    combat.enemy_party = party.clone();
    combat.player_used_moves = vec!["THUNDERBOLT".into()];
    combat.enemy_spikes = true;
    combat.enemy_reflect_turns = 4;
    let player_hp = combat.player.hp;
    state.script_runtime.active_battle_combat = Some(combat);
    // Selection must still pass the ordinary reward gate before publishing.
    let before = state.clone();
    assert!(data().advance_active_trainer_battle(&mut state).is_err());
    assert_eq!(state, before);
    state.battle_rewarded_enemy_party_indices.insert(0);
    let before_replacement = state.clone();
    assert!(
        crystal_core::battle::start::advance_active_trainer_battle_with_replacement(
            &mut state,
            Some(99)
        )
        .is_err()
    );
    assert_eq!(
        state, before_replacement,
        "invalid replacement must be atomic"
    );
    let mut stock = data().clone();
    stock.competitive_battles = false;
    let mut original_order = state.clone();
    assert_eq!(
        stock
            .advance_active_trainer_battle(&mut original_order)
            .unwrap()
            .next_enemy
            .unwrap()
            .species
            .id,
        "GYARADOS"
    );
    let outcome = data().advance_active_trainer_battle(&mut state).unwrap();
    assert_eq!(outcome.next_enemy.unwrap().species.id, "DONPHAN");
    assert_eq!(state.battle_active_enemy_party_index, Some(2));
    let combat = state.script_runtime.active_battle_combat.as_ref().unwrap();
    assert_eq!(combat.turn, 0);
    assert_eq!(combat.player.hp, player_hp);
    assert_eq!(combat.enemy.hp, party[2].max_hp - party[2].max_hp / 8);
    assert_eq!(combat.enemy_reflect_turns, 4);
    assert_eq!(combat.enemy.stat_boosts[&Stat::Attack], 0);
    assert_eq!(combat.enemy_party[1], party[1]);
    assert_eq!(combat.enemy_party[0], party[0]);
    assert_eq!(
        state.battle_rewarded_enemy_party_indices,
        std::collections::BTreeSet::from([0])
    );
    assert!(state.pokedex.has_seen("DONPHAN"));
}

#[test]
#[ignore = "explicit pack-backed AI strength evaluation"]
fn competitive_ai_strength_against_original_champion_policy() {
    run_strength_suite("CHAMPION");
}

#[test]
#[ignore = "explicit 288-battle trainer-profile strength evaluation"]
fn competitive_ai_strength_across_trainer_profiles() {
    std::thread::scope(|scope| {
        let workers = PROFILE_CLASSES.map(|class| scope.spawn(move || run_strength_suite(class)));
        for worker in workers {
            worker.join().unwrap();
        }
    });
}

fn run_strength_suite(class: &str) {
    let mut original = data().clone();
    original.competitive_battles = false;
    let champion = original
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "CHAMPION")
        .unwrap();
    let flags = champion.ai_move_flags;
    let cases: &[(&str, &[&str])] = &[
        (
            "ALAKAZAM",
            &["PSYCHIC_M", "SHADOW_BALL", "THUNDERPUNCH", "RECOVER"],
        ),
        (
            "GENGAR",
            &["SHADOW_BALL", "HYPNOSIS", "THUNDERBOLT", "GIGA_DRAIN"],
        ),
        ("MACHAMP", &["CROSS_CHOP", "ROCK_SLIDE", "CURSE", "REST"]),
        (
            "TYPHLOSION",
            &[
                "FLAMETHROWER",
                "THUNDERPUNCH",
                "QUICK_ATTACK",
                "SMOKESCREEN",
            ],
        ),
        ("BLASTOISE", &["SURF", "ICE_BEAM", "BITE", "PROTECT"]),
        (
            "MEGANIUM",
            &["RAZOR_LEAF", "SYNTHESIS", "REFLECT", "BODY_SLAM"],
        ),
    ];
    let (mut wins, mut losses, mut draws) = (0, 0, 0);
    for (species, moves) in cases {
        let before = (wins, losses, draws);
        for seed in 1..=8 {
            let p = pokemon(species, moves);
            let mut state = BattleCombatState::new(p.clone(), p);
            // Both policies have the same public information and move split.
            state.player_used_moves = moves.iter().map(|m| m.to_string()).collect();
            let mut battle_rng = Random(seed * 7_919);
            let mut stock_rng = Random(seed * 103);
            let mut planner_rng = Random(seed * 107);
            let mut opening = Vec::new();
            for turn in 0..32 {
                let json = serde_json::to_value(&state).unwrap();
                let mut reversed = json.clone();
                for (key, value) in json.as_object().unwrap() {
                    let other = if key == "player" {
                        "enemy".into()
                    } else if key == "enemy" {
                        "player".into()
                    } else if let Some(tail) = key.strip_prefix("player_") {
                        format!("enemy_{tail}")
                    } else if let Some(tail) = key.strip_prefix("enemy_") {
                        format!("player_{tail}")
                    } else {
                        continue;
                    };
                    if json.get(&other).is_some() {
                        reversed[other] = value.clone();
                    }
                }
                let mut reversed: BattleCombatState = serde_json::from_value(reversed).unwrap();
                reversed.player_used_moves = moves.iter().map(|m| m.to_string()).collect();
                let player_slot = original
                    .select_trainer_enemy_move_slot(&reversed, flags, &mut stock_rng)
                    .unwrap();
                let enemy_slot = data()
                    .select_trainer_enemy_move_for_class(&state, class, 0, &mut planner_rng)
                    .unwrap()
                    .slot;
                if seed == 1 && turn < 4 {
                    opening.push(format!(
                        "{}-{} HP {}/{}",
                        moves[enemy_slot], moves[player_slot], state.enemy.hp, state.player.hp
                    ));
                }
                state = data()
                    .resolve_battle_turn_with_items(
                        state,
                        BattleTurnInput {
                            player: BattleAction::Move { slot: player_slot },
                            enemy: BattleAction::Move { slot: enemy_slot },
                        },
                        &mut battle_rng,
                    )
                    .unwrap()
                    .state;
                if state.player.hp == 0 || state.enemy.hp == 0 {
                    break;
                }
            }
            match (state.player.hp == 0, state.enemy.hp == 0) {
                (true, false) => wins += 1,
                (false, true) => losses += 1,
                _ => draws += 1,
            }
            if seed == 1 {
                eprintln!("{species} sample opening: {opening:?}");
            }
        }
        eprintln!(
            "{class} {species}: competitive {}, original {}, draws {}",
            wins - before.0,
            losses - before.1,
            draws - before.2
        );
    }
    eprintln!(
        "{class}: 48 equal-stat mirrored singles vs original Champion flags {flags:#x}: competitive {wins}, original {losses}, draws {draws}"
    );
    assert!(
        (wins * 2 + draws) * 100 >= 48 * 2 * 70,
        "every profile must score at least 70% against the original Champion policy"
    );
}

#[test]
fn competitive_split_drives_screens_counter_and_mirror_coat_by_move_category() {
    use crystal_core::battle::turn::{BattleEvent, BattleSide, preview_battle_move};
    let d = data();
    let mut state = BattleCombatState::new(
        pokemon("ALAKAZAM", &["FIRE_PUNCH", "SHADOW_BALL"]),
        pokemon("BLASTOISE", &["COUNTER", "MIRROR_COAT"]),
    );
    let max_damage = |state: &BattleCombatState, id: &str| {
        preview_battle_move(
            state,
            BattleSide::Player,
            &d.moves[id],
            &d.items,
            &d.move_priorities,
            &d.battle_stat_multipliers,
            &d.type_categories,
            &d.type_effectiveness,
            &d.weather_modifiers,
        )
        .unwrap()
        .damage_on_hit
        .unwrap()[1]
    };
    let punch = max_damage(&state, "FIRE_PUNCH");
    let shadow = max_damage(&state, "SHADOW_BALL");
    state.enemy_reflect_turns = 5;
    assert!(max_damage(&state, "FIRE_PUNCH") < punch);
    assert_eq!(max_damage(&state, "SHADOW_BALL"), shadow);
    state.enemy_reflect_turns = 0;
    state.enemy_light_screen_turns = 5;
    assert_eq!(max_damage(&state, "FIRE_PUNCH"), punch);
    assert!(max_damage(&state, "SHADOW_BALL") < shadow);
    state.enemy_light_screen_turns = 0;
    for (attacking_slot, responding_slot, should_counter) in
        [(0, 0, true), (0, 1, false), (1, 0, false), (1, 1, true)]
    {
        let outcome = d
            .resolve_battle_turn_with_items(
                state.clone(),
                BattleTurnInput {
                    player: BattleAction::Move {
                        slot: attacking_slot,
                    },
                    enemy: BattleAction::Move {
                        slot: responding_slot,
                    },
                },
                &mut Random(42),
            )
            .unwrap();
        assert!(
            outcome.events.iter().any(|e| matches!(
                e,
                BattleEvent::Damage {
                    side: BattleSide::Player,
                    ..
                }
            )),
            "fixture attack must land"
        );
        let countered = outcome.events.iter().any(|e| {
            matches!(
                e,
                BattleEvent::CounterDamage {
                    side: BattleSide::Enemy,
                    ..
                }
            )
        });
        assert_eq!(
            countered, should_counter,
            "attack {attacking_slot}, response {responding_slot}"
        );
    }
}

#[test]
fn competitive_split_uses_gyarados_attack_and_gengar_special_attack_without_changing_stats() {
    use crystal_core::battle::turn::{BattleSide, preview_battle_move};
    let d = data();
    let mut classic_categories = d.type_categories.clone();
    classic_categories.moves.clear();
    for (species, move_id) in [("GYARADOS", "WATERFALL"), ("GENGAR", "SHADOW_BALL")] {
        let attacker = pokemon(species, &[move_id]);
        assert!(attacker.species.tmhm_learnset.iter().any(|m| m == move_id));
        let state = BattleCombatState::new(attacker, pokemon("BLASTOISE", &["TACKLE"]));
        let damage = |categories| {
            preview_battle_move(
                &state,
                BattleSide::Player,
                &d.moves[move_id],
                &d.items,
                &d.move_priorities,
                &d.battle_stat_multipliers,
                categories,
                &d.type_effectiveness,
                &d.weather_modifiers,
            )
            .unwrap()
            .damage_on_hit
            .unwrap()[1]
        };
        let split = damage(&d.type_categories);
        let classic = damage(&classic_categories);
        assert!(
            split > classic,
            "{species} {move_id}: split {split}, classic {classic}"
        );
    }
}
