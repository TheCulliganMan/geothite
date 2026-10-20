use std::{collections::BTreeSet, path::PathBuf, sync::OnceLock};

use crystal_assets::{
    CompiledGamePack, WildChallengeRules, build_wild_challenge_modpack,
    read_verified_compiled_game_pack,
};
use crystal_core::{
    battle::{
        capture::CaptureOutcome,
        turn::{BattleAction, BattleCombatState},
    },
    models::{Dv, Pokemon},
    random::{BattleRandomSource, CrystalRandom, ReplayDivider},
    state::{BattleMemory, GameState},
    world::{encounters::EncounterSurface, map::TilePosition},
};

fn packs() -> &'static (CompiledGamePack, CompiledGamePack) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_WILD_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let path = path.canonicalize().expect("resolve external base pack");
        let base = read_verified_compiled_game_pack(path).expect("verified external base pack");
        let pack = build_wild_challenge_modpack(&base).unwrap();
        (base, pack)
    })
}

fn mon(id: &str, level: u8) -> Pokemon {
    packs()
        .1
        .data()
        .create_pokemon(id, level, Dv::from_non_hp(10, 10, 10, 10))
        .unwrap()
}

fn can_know_move(
    data: &crystal_assets::GameDataSet,
    species: &str,
    level: u8,
    move_id: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if !visited.insert(species.to_string()) {
        return false;
    }
    if data.learnsets[species]
        .iter()
        .any(|entry| entry.0 <= level && entry.1 == move_id)
        || data.pokemon[species]
            .tmhm_learnset
            .iter()
            .any(|id| id == move_id)
        || data
            .egg_moves
            .get(species)
            .and_then(|value| value.as_array())
            .is_some_and(|moves| moves.iter().any(|value| value.as_str() == Some(move_id)))
    {
        return true;
    }
    data.evolutions.0.iter().any(|(parent, evolutions)| {
        evolutions.iter().any(|entry| entry.species == species)
            && can_know_move(data, parent, level, move_id, visited)
    })
}

#[test]
fn starters_receive_the_same_expanded_moves_as_wild_pokemon() {
    use crystal_core::systems::gift_pokemon::GiftPokemonRequest;
    for id in ["CHIKORITA", "CYNDAQUIL", "TOTODILE"] {
        let script = match id {
            "CHIKORITA" => "ChikoritaPokeBallScript",
            "CYNDAQUIL" => "CyndaquilPokeBallScript",
            _ => "TotodilePokeBallScript",
        };
        for enabled in [false, true] {
            let mut data = packs().1.data().clone();
            if !enabled {
                data.wild_challenge = None;
            }
            let mut state = GameState::reset_wram_for_new_game();
            let pokemon = data
                .create_pokemon(id, 5, Dv::from_non_hp(10, 10, 10, 10))
                .unwrap();
            let expected = if enabled {
                data.competitive_wild_moves(&pokemon).unwrap()
            } else {
                pokemon.moves.clone()
            };
            let outcome = data
                .grant_gift_pokemon_to_state(
                    &mut state,
                    GiftPokemonRequest {
                        species_id: id.into(),
                        level: 5,
                        held_item_id: None,
                        nickname: None,
                        original_trainer_name: "CHRIS".into(),
                        original_trainer_id: 1234,
                        caught_data: None,
                        source_script: script.into(),
                        command_index: 0,
                        egg: false,
                        dvs: Dv::from_non_hp(10, 10, 10, 10),
                    },
                )
                .unwrap();
            assert_eq!(outcome.pokemon.moves, expected, "{id} enabled={enabled}");
            assert_eq!(
                state.storage.party.pokemon[0].as_ref().unwrap().moves,
                expected
            );
            if enabled {
                assert_ne!(expected, pokemon.moves, "{id} should gain expanded moves");
            }
        }
    }
}

#[test]
fn wild_challenge_pack_roundtrips_without_changing_content_or_audio() {
    let (base, pack) = packs();
    let mut unchanged = pack.data().clone();
    unchanged.wild_challenge = base.data().wild_challenge;
    assert_eq!(&unchanged, base.data());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().unwrap(),
        base.audio_manifest().unwrap()
    );
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(
        pack.identity().unwrap() == base.identity().unwrap(),
        base.data().wild_challenge.is_some()
    );
    assert_eq!(
        pack.identity().unwrap(),
        build_wild_challenge_modpack(base)
            .unwrap()
            .identity()
            .unwrap()
    );
    assert_eq!(
        build_wild_challenge_modpack(pack)
            .unwrap()
            .identity()
            .unwrap(),
        pack.identity().unwrap()
    );
    let path =
        std::env::temp_dir().join(format!("wild-challenge-{}.crystalpack", std::process::id()));
    pack.write_preserving_storage(&path).unwrap();
    let loaded = read_verified_compiled_game_pack(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.data(), pack.data());
    assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
}

#[test]
fn wild_challenge_scales_with_team_and_preserves_floors_and_bounds() {
    let rules = WildChallengeRules::default();
    let mut party = vec![
        mon("TYPHLOSION", 50),
        mon("AMPHAROS", 40),
        mon("QUAGSIRE", 30),
    ];
    let levels: Vec<_> = (0..5)
        .map(|variation| rules.encounter_level(&party, 3, variation))
        .collect();
    assert_eq!(levels, [37, 38, 39, 40, 41]);
    party.push(mon("MAGIKARP", 5));
    party[0].hp = 0;
    assert_eq!(rules.encounter_level(&party, 3, 2), 39);
    party.reverse();
    assert_eq!(rules.encounter_level(&party, 3, 2), 39);
    let mut egg = mon("TOGEPI", 100);
    egg.is_egg = true;
    party.push(egg);
    assert_eq!(rules.encounter_level(&party, 3, 2), 39);
    assert_eq!(rules.encounter_level(&party, 70, 2), 70);
    assert_eq!(rules.encounter_level(&[], 3, 2), 3);
    assert_eq!(rules.encounter_level(&[mon("MEW", 1)], 1, 0), 1);
    assert_eq!(rules.encounter_level(&[mon("MEW", 100)], 3, 4), 100);
    assert_eq!(rules.encounter_level(&[mon("MEW", 5)], 3, 2), 4);
    assert_eq!(rules.encounter_level(&[mon("MEW", 80)], 3, 2), 79);
}

#[test]
fn wild_challenge_composes_with_the_modern_move_split() {
    let split = if packs().0.data().type_categories.moves.is_empty() {
        crystal_assets::build_modern_move_split_modpack(&packs().0).unwrap()
    } else {
        packs().0.clone()
    };
    let challenge = build_wild_challenge_modpack(&split).unwrap();
    assert_eq!(
        challenge.data().type_categories,
        split.data().type_categories
    );
    let pokemon = challenge
        .data()
        .create_pokemon("ALAKAZAM", 50, Dv::from_non_hp(10, 10, 10, 10))
        .unwrap();
    let moves = challenge.data().competitive_wild_moves(&pokemon).unwrap();
    assert!(
        moves.iter().any(|m| m.name == "SHADOW_BALL"),
        "the special Shadow Ball should suit Alakazam: {moves:?}"
    );
}

#[test]
fn wild_challenge_sets_are_legal_and_sane_for_every_species_and_level_band() {
    let data = packs().1.data();
    for id in data.pokemon.keys() {
        for level in [5, 15, 25, 50, 100] {
            let pokemon = mon(id, level);
            let moves = data.competitive_wild_moves(&pokemon).unwrap();
            assert!(!moves.is_empty() && moves.len() <= 4, "{id} lv{level}");
            assert_eq!(
                moves.len(),
                moves.iter().map(|m| &m.name).collect::<BTreeSet<_>>().len()
            );
            for learned in &moves {
                assert!(
                    can_know_move(data, id, level, &learned.name, &mut BTreeSet::new()),
                    "{id}: {}",
                    learned.name
                );
                assert_eq!(learned.current_pp, data.moves[&learned.name].pp);
                assert_eq!(learned.pp_ups, 0);
            }
            assert_eq!(moves, data.competitive_wild_moves(&pokemon).unwrap());
        }
    }
    for id in [
        "ALAKAZAM",
        "GENGAR",
        "SNORLAX",
        "STARMIE",
        "TYRANITAR",
        "SCIZOR",
        "MAGIKARP",
        "DITTO",
        "UNOWN",
    ] {
        let moves = data.competitive_wild_moves(&mon(id, 50)).unwrap();
        eprintln!(
            "{id}: {:?}",
            moves.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }
    let alakazam = data.competitive_wild_moves(&mon("ALAKAZAM", 50)).unwrap();
    assert!(alakazam.iter().any(|m| m.name == "PSYCHIC_M"));
    assert!(alakazam.iter().any(|m| m.name == "RECOVER"));
    let skarmory = data.competitive_wild_moves(&mon("SKARMORY", 50)).unwrap();
    assert!(
        skarmory.iter().any(|m| m.name == "DRILL_PECK"),
        "legal egg coverage must be available: {skarmory:?}"
    );
    let arcanine = data.competitive_wild_moves(&mon("ARCANINE", 50)).unwrap();
    assert!(
        arcanine
            .iter()
            .any(|m| m.name == "FLAMETHROWER" || m.name == "FIRE_BLAST")
    );
    let ditto = data.competitive_wild_moves(&mon("DITTO", 50)).unwrap();
    assert_eq!(
        ditto.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
        ["TRANSFORM"]
    );
    let unown = data.competitive_wild_moves(&mon("UNOWN", 50)).unwrap();
    assert_eq!(
        unown.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
        ["HIDDEN_POWER"]
    );
}

fn roundtrip_state(state: &GameState) -> GameState {
    use crystal_core::save::{
        SaveModpackIdentity, read_save_game_for_modpack, write_save_game_for_modpack,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_SAVE: AtomicUsize = AtomicUsize::new(0);
    static IDENTITY: OnceLock<(SaveModpackIdentity, String)> = OnceLock::new();
    let (modpack, content_hash) = IDENTITY.get_or_init(|| {
        let path = std::env::temp_dir().join(format!(
            "wild-challenge-save-identity-{}.crystalpack",
            std::process::id()
        ));
        packs().1.write_preserving_storage(&path).unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        (
            loaded.save_modpack_identity().unwrap(),
            loaded.pack_content_hash().unwrap(),
        )
    });
    let path = std::env::temp_dir().join(format!(
        "wild-challenge-state-{}-{}.crystalsave",
        std::process::id(),
        NEXT_SAVE.fetch_add(1, Ordering::Relaxed)
    ));
    write_save_game_for_modpack(&path, state.clone(), modpack, content_hash)
        .expect("write production binary save");
    let loaded = read_save_game_for_modpack(&path, modpack, content_hash);
    std::fs::remove_file(&path).expect("remove temporary save");
    loaded
        .expect("reload validated production binary save")
        .into_state()
}

fn new_state() -> GameState {
    let data = packs().1.data();
    let mut state = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).unwrap(),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .unwrap()
        .0;
    state.player_name = "CHRIS".into();
    state
}

fn enter_session(
    state: &mut GameState,
    mut session: crystal_core::world::session::OverworldSession,
) -> crystal_core::world::session::OverworldSession {
    let data = packs().1.data();
    let map_name = session.map.name.clone();
    let tile = session.player.tile;
    let facing = session.player.facing;
    data.transition_overworld_session(
        state,
        &mut session,
        &map_name,
        tile,
        crystal_core::systems::map_context::SpawnMemoryUpdate::Preserve,
        &data.audio_ids().into_iter().map(str::to_owned).collect(),
    )
    .expect("enter fixture map through production map setup");
    session.player.facing = facing;
    data.commit_overworld_snapshot(
        state,
        &session,
        crystal_core::systems::map_context::SpawnMemoryUpdate::Preserve,
    );
    session
}

fn grass_session(state: &mut GameState) -> crystal_core::world::session::OverworldSession {
    let data = packs().1.data();
    *state = new_state();
    let (width, height) = data.saved_map_tile_bounds("Route29").unwrap();
    for y in 0..height {
        for x in 0..width {
            if let Ok(session) = data.overworld_session(
                "Route29",
                TilePosition {
                    x: x as i16,
                    y: y as i16,
                },
                0,
            ) {
                if session
                    .current_encounter_surface_checked_with_land_encounters(false)
                    .unwrap()
                    == Some(EncounterSurface::Grass)
                {
                    return enter_session(state, session);
                }
            }
        }
    }
    panic!("Route29 must contain grass");
}

#[test]
fn wild_challenge_real_encounter_generates_scaled_stats_moves_and_capture() {
    let data = packs().1.data();
    let mut state = GameState::reset_wram_for_new_game();
    let session = grass_session(&mut state);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 50))
        .unwrap();
    state.sync_party_from_storage();
    let mut divider = ReplayDivider::new((0..=255).cycle().take(100_000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let roll = (0..500)
        .find_map(|_| {
            data.check_wild_encounter_after_step(&mut state, &session, &mut rng)
                .unwrap()
                .filter(|roll| roll.resolved.is_some())
        })
        .expect("actual grass chooser produced an encounter");
    let level = roll.resolved.as_ref().unwrap().level;
    assert!((47..=51).contains(&level));
    let battle = data
        .start_resolved_wild_encounter_after_step(&mut state, &session, &Some(roll), &mut rng)
        .unwrap()
        .unwrap();
    state.random_state = rng.state();
    let enemy = &battle.enemy_pokemon;
    assert_eq!(enemy.level, level);
    let expected = data
        .create_pokemon(&enemy.species.id, level, enemy.dvs)
        .unwrap();
    assert_eq!(
        (enemy.attack, enemy.max_hp, enemy.experience),
        (expected.attack, expected.max_hp, expected.experience)
    );
    assert_eq!(enemy.moves, data.competitive_wild_moves(enemy).unwrap());
    data.validate_saved_wild_battle_origin_references(&battle.battle_type, "Route29", enemy)
        .unwrap();
    let mut legacy = packs().0.data().clone();
    legacy.wild_challenge = None;
    assert!(
        legacy
            .validate_saved_wild_battle_origin_references(&battle.battle_type, "Route29", enemy)
            .is_err()
    );
    assert!(!data.saved_wild_encounter_exists("Route29", "MEWTWO", level));
    let reloaded: GameState = roundtrip_state(&state);
    assert_eq!(reloaded.battle, state.battle);
    let outcome = CaptureOutcome {
        caught: true,
        blocked: false,
        storage_full: false,
        wobble_count: 4,
        animation_shakes: 3,
        final_catch_rate: 255,
        ball_id: None,
    };
    let captured = data
        .complete_active_wild_capture(&mut state, &outcome, None, &mut divider)
        .unwrap()
        .stored
        .unwrap()
        .pokemon;
    assert_eq!(captured.level, enemy.level);
    assert_eq!(captured.moves, enemy.moves);
    data.validate_saved_storage_references(&state.storage)
        .unwrap();
    assert!(matches!(state.battle, BattleMemory::Inactive));
}

#[test]
fn wild_challenge_scripted_battle_scales_and_retains_exact_script_origin() {
    let data = packs().1.data();
    let mut state = new_state();
    let session = data
        .overworld_session("Route36", TilePosition::new(20, 8), 0)
        .unwrap();
    let mut session = enter_session(&mut state, session);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 50))
        .unwrap();
    state.sync_party_from_storage();
    let mut divider = ReplayDivider::new([0; 64]);
    let battle = data
        .start_scripted_wild_battle(
            &mut state,
            "Route36",
            "Route36",
            "WateredWeirdTreeScript",
            12,
            &mut divider,
        )
        .unwrap();
    assert_eq!(battle.level, 49);
    assert_eq!(battle.enemy_pokemon.level, 49);
    assert_eq!(
        battle.enemy_pokemon.moves,
        data.competitive_wild_moves(&battle.enemy_pokemon).unwrap()
    );
    assert!(data.saved_static_wild_battle_origin_exists(
        "Route36",
        "WateredWeirdTreeScript",
        12,
        13,
        &battle.battle_type,
        "SUDOWOODO",
        49
    ));
    assert!(!data.saved_static_wild_battle_origin_exists(
        "Route36",
        "WateredWeirdTreeScript",
        12,
        13,
        &battle.battle_type,
        "MEWTWO",
        49
    ));
    let _: GameState = roundtrip_state(&state);
    let origin = crystal_assets::RuntimeStaticWildBattleOrigin {
        map_name: "Route36".into(),
        source_script: "WateredWeirdTreeScript".into(),
        startbattle_command_index: 12,
        resume_command_index: 13,
        battle_type: battle.battle_type.clone(),
        species: "SUDOWOODO".into(),
        level: battle.level,
    };
    let caught = CaptureOutcome {
        caught: true,
        blocked: false,
        storage_full: false,
        wobble_count: 4,
        animation_shakes: 3,
        final_catch_rate: 255,
        ball_id: None,
    };
    let completion = data
        .complete_active_wild_capture(&mut state, &caught, None, &mut divider)
        .unwrap();
    assert_eq!(completion.stored.unwrap().pokemon.level, 49);
    let terminal = data.scripted_wild_battle_terminal(&state, &origin).unwrap();
    data.complete_scripted_wild_battle_in_session(
        &mut state,
        &mut session,
        &origin,
        terminal,
        &mut divider,
    )
    .unwrap();
    assert!(state.pending_static_wild_terminal.is_none());
}

#[derive(Default)]
struct FixedRandom(u8);
impl BattleRandomSource for FixedRandom {
    fn battle_random_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_add(97);
        self.0
    }
}

#[test]
fn wild_challenge_scorer_avoids_immune_attacks_and_spends_real_pp() {
    let data = packs().1.data();
    let mut player = mon("GENGAR", 50);
    player.moves.retain(|m| m.name == "NIGHT_SHADE");
    assert_eq!(player.moves.len(), 1);
    let mut enemy = mon("TAUROS", 50);
    enemy.moves = ["TACKLE", "EARTHQUAKE"]
        .into_iter()
        .map(|id| crystal_core::models::LearnedMove {
            name: id.to_string(),
            current_pp: data.moves[id].pp,
            pp_ups: 0,
        })
        .collect();
    let combat = BattleCombatState::new(player, enemy);
    let selected = data
        .select_challenge_wild_move(&combat, &mut FixedRandom::default())
        .unwrap();
    assert_eq!(selected.slot, 1);
    let before = combat.enemy.moves[1].current_pp;
    let outcome = data
        .resolve_wild_battle_turn_with_items(
            combat,
            crystal_core::battle::turn::BattleTurnInput {
                player: BattleAction::Move { slot: 0 },
                enemy: BattleAction::Move {
                    slot: selected.slot,
                },
            },
            0,
            &mut FixedRandom::default(),
        )
        .unwrap();
    assert_eq!(outcome.state.enemy.moves[1].current_pp, before - 1);
}

#[test]
fn default_wild_ai_replays_the_live_choice_and_rejects_a_forged_move_atomically() {
    use crystal_assets::{RuntimeBattleTurnCommand, RuntimeDividerTrace, RuntimeMutationCommand};
    use crystal_core::{
        battle::turn::{BattleTurnError, EnemyMoveSelection},
        random::RecordingDivider,
    };
    use std::cell::Cell;
    struct Counting<'a> {
        rng: &'a mut dyn BattleRandomSource,
        calls: &'a Cell<u16>,
    }
    impl BattleRandomSource for Counting<'_> {
        fn battle_random_byte(&mut self) -> u8 {
            self.calls.set(self.calls.get() + 1);
            self.rng.battle_random_byte()
        }
    }
    let pack = crystal_assets::build_default_battle_modpack(&packs().1).unwrap();
    let data = pack.data();
    assert!(data.competitive_battles && data.wild_challenge.is_some());
    let mut state = new_state();
    let session = data
        .overworld_session("Route36", TilePosition::new(20, 8), 0)
        .unwrap();
    let mut session = enter_session(&mut state, session);
    let mut player = mon("GENGAR", 50);
    player.moves.retain(|m| m.name == "NIGHT_SHADE");
    state.storage.register_capture_in_box(0, player).unwrap();
    state.sync_party_from_storage();
    let battle = data
        .start_scripted_wild_battle(
            &mut state,
            "Route36",
            "Route36",
            "WateredWeirdTreeScript",
            12,
            &mut ReplayDivider::new([0; 64]),
        )
        .unwrap();
    let mut combat = crystal_core::battle::turn::active_battle_combat_state(&state).unwrap();
    combat.enemy.moves = ["TACKLE", "ROCK_THROW"]
        .into_iter()
        .map(|name| crystal_core::models::LearnedMove {
            name: name.into(),
            current_pp: data.moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    combat.enemy_party[combat.enemy_party_index] = combat.enemy.clone();
    if let BattleMemory::StaticWild {
        enemy_pokemon,
        enemy_party,
        ..
    } = &mut state.battle
    {
        *enemy_pokemon = combat.enemy.clone();
        enemy_party[combat.enemy_party_index] = combat.enemy.clone();
    } else {
        panic!("expected authored static wild encounter");
    }
    let pp = combat.enemy.moves[1].current_pp;
    state.script_runtime.active_battle_combat = Some(combat);
    let before = state.clone();
    let mut expected = state.clone();
    let calls = Cell::new(0);
    let selected = Cell::new(None);
    let mut move_selector = |combat: &BattleCombatState,
                             rng: &mut dyn BattleRandomSource|
     -> Result<EnemyMoveSelection, BattleTurnError> {
        let selection = data
            .select_wild_enemy_move_for_battle(
                combat,
                &battle.battle_type,
                &mut Counting { rng, calls: &calls },
            )
            .map_err(|error| BattleTurnError::EnemyActionSelectionFailed {
                error: error.to_string(),
            })?;
        selected.set(Some(selection.slot));
        Ok(selection)
    };
    let mut action_selector = |_: &BattleCombatState, _: &mut dyn BattleRandomSource| {
        Ok(BattleAction::Move {
            slot: selected.get().unwrap(),
        })
    };
    let mut recording = RecordingDivider::new(ReplayDivider::new((0..8192).map(|i| i as u8)));
    data.resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
        &mut expected,
        BattleAction::Move { slot: 0 },
        &mut recording,
        &mut move_selector,
        &mut action_selector,
    )
    .unwrap();
    assert_eq!(
        selected.get(),
        Some(1),
        "wild AI must avoid the immune Normal attack"
    );
    assert_eq!(
        expected
            .script_runtime
            .active_battle_combat
            .as_ref()
            .unwrap()
            .enemy
            .moves[1]
            .current_pp,
        pp - 1
    );
    let command = RuntimeBattleTurnCommand {
        player_action: BattleAction::Move { slot: 0 },
        player_bag_item_id: None,
        enemy_action: BattleAction::Move { slot: 1 },
        enemy_ai_divider_trace: RuntimeDividerTrace::new([]),
        enemy_ai_selected_move_slot: Some(1),
        enemy_move_ai_random_calls: calls.get(),
        enemy_post_order_ai_random_calls: 0,
        divider_trace: RuntimeDividerTrace::new(recording.samples().iter().copied()),
    };
    let ids = data.audio_ids().into_iter().map(str::to_owned).collect();
    let mut forged = command.clone();
    forged.enemy_ai_selected_move_slot = Some(0);
    forged.enemy_action = BattleAction::Move { slot: 0 };
    assert!(
        data.apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::ResolveActiveBattleTurn(forged),
            &ids,
            &ids,
            &ids
        )
        .is_err()
    );
    assert_eq!(
        state, before,
        "a forged choice must leave authoritative state unchanged"
    );
    data.apply_runtime_mutation_command(
        &mut state,
        &mut session,
        RuntimeMutationCommand::ResolveActiveBattleTurn(command),
        &ids,
        &ids,
        &ids,
    )
    .unwrap();
    assert_eq!(
        state, expected,
        "live and replay must execute the same wild turn"
    );
}

#[test]
fn shared_wild_selector_preserves_tutorial_and_contest_ai() {
    let pack = crystal_assets::build_default_battle_modpack(&packs().1).unwrap();
    let state = BattleCombatState::new(mon("GENGAR", 50), mon("TAUROS", 50));
    for battle_type in ["BATTLETYPE_TUTORIAL", "BATTLETYPE_CONTEST"] {
        let expected = crystal_core::battle::turn::select_wild_enemy_move_slot(
            &state,
            &mut FixedRandom::default(),
        );
        let selected = pack
            .data()
            .select_wild_enemy_move_for_battle(&state, battle_type, &mut FixedRandom::default())
            .unwrap();
        assert_eq!(selected.slot, expected);
        assert_eq!(selected.ai_damage_register, 0);
    }
}

#[test]
fn wild_challenge_repel_checks_scaled_levels() {
    let data = packs().1.data();
    let mut state = new_state();
    let session = grass_session(&mut state);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 50))
        .unwrap();
    state.sync_party_from_storage();
    state.repel_steps_remaining = 100;
    state.active_repel_item = Some("REPEL".into());
    let mut divider = ReplayDivider::new((0..=255).cycle().take(100_000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let (mut blocked, mut allowed) = (false, false);
    for _ in 0..1000 {
        if let Some(roll) = data
            .check_wild_encounter_after_step(&mut state, &session, &mut rng)
            .unwrap()
        {
            blocked |= roll.repelled_by.as_deref() == Some("REPEL");
            if let Some(resolved) = roll.resolved {
                assert!(resolved.level >= 50);
                allowed = true;
            }
        }
        if blocked && allowed {
            break;
        }
    }
    assert!(
        blocked && allowed,
        "Repel must allow scaled peers as well as block lower scaled levels"
    );
}

#[test]
fn wild_challenge_roamer_keeps_dvs_damage_and_save_origin_when_scaling() {
    let data = packs().1.data();
    let mut state = new_state();
    let session = grass_session(&mut state);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 80))
        .unwrap();
    state.sync_party_from_storage();
    let init = data.roaming_pokemon.init_write(0).unwrap();
    let metadata = data.runtime_map_metadata_for_name("Route29").unwrap();
    state.roaming_pokemon[0].species = Some(init.species.clone());
    state.roaming_pokemon[0].level = init.level;
    state.roaming_pokemon[0].map_group = metadata.group_id as u8;
    state.roaming_pokemon[0].map_number = metadata.map_id as u8;
    state.roaming_pokemon[0].dvs_be = [0xaa, 0xaa];
    state.roaming_pokemon[0].hp = 50;
    let old = mon(&init.species, init.level);
    let mut divider = ReplayDivider::new((0..=255).cycle().take(100_000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let roll = (0..1000)
        .find_map(|_| {
            data.check_wild_encounter_after_step(&mut state, &session, &mut rng)
                .unwrap()
                .filter(|roll| roll.roaming_slot == Some(0) && roll.resolved.is_some())
        })
        .expect("real roaming chooser");
    let battle = data
        .start_resolved_wild_encounter_after_step(&mut state, &session, &Some(roll), &mut rng)
        .unwrap()
        .unwrap();
    let enemy = battle.enemy_pokemon;
    assert_eq!(enemy.level, 79);
    assert_eq!(enemy.dvs, old.dvs);
    assert_eq!(
        enemy.hp,
        (50 * u32::from(enemy.max_hp) / u32::from(old.max_hp)).clamp(1, 255) as u16
    );
    data.validate_saved_roaming_references(&state.roaming_pokemon, &state.roaming_map_history)
        .unwrap();
    data.validate_saved_roaming_battle_origin_references(
        "Route29",
        0,
        &state.roaming_pokemon[0],
        &enemy,
    )
    .unwrap();
    let _: GameState = roundtrip_state(&state);
}

#[test]
fn wild_challenge_fishing_uses_scaled_materialization() {
    use crystal_core::world::{
        collision::{Terrain, describe_collision, sample_collision},
        map::Direction,
    };
    let data = packs().1.data();
    let (width, height) = data.saved_map_tile_bounds("NewBarkTown").unwrap();
    let mut shore = None;
    'tiles: for y in 0..height {
        for x in 0..width {
            if let Ok(mut session) =
                data.overworld_session("NewBarkTown", TilePosition::new(x as i16, y as i16), 0)
            {
                for facing in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    if sample_collision(
                        &session.map,
                        &session.tileset,
                        session.player.tile.moved(facing),
                    )
                    .is_some_and(|sample| {
                        describe_collision(sample.permission).terrain == Terrain::Water
                    }) {
                        session.player.facing = facing;
                        shore = Some(session);
                        break 'tiles;
                    }
                }
            }
        }
    }
    let session = shore.expect("New Bark fishing shore");
    let mut state = new_state();
    let session = enter_session(&mut state, session);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 50))
        .unwrap();
    state.sync_party_from_storage();
    let mut divider = ReplayDivider::new((0..=255).cycle().take(100_000));
    let battle = (0..100)
        .find_map(|_| {
            data.cast_fishing_rod_in_session_with_divider(
                &mut state,
                &session,
                crystal_core::world::fishing::ROD_OLD,
                &mut divider,
            )
            .unwrap()
            .wild_battle
        })
        .expect("successful fishing cast");
    assert!((47..=51).contains(&battle.enemy_pokemon.level));
    assert_eq!(
        battle.enemy_pokemon.moves,
        data.competitive_wild_moves(&battle.enemy_pokemon).unwrap()
    );
    data.validate_saved_wild_battle_origin_references(
        &battle.battle_type,
        "NewBarkTown",
        &battle.enemy_pokemon,
    )
    .unwrap();
    let _: GameState = roundtrip_state(&state);
}

#[test]
fn wild_challenge_fresh_high_level_roamer_preserves_hp_above_a_byte() {
    use crystal_core::systems::roaming::{
        RoamingBattleEndInput, RoamingEngineState, battle_end_handle_roam_mons,
    };
    let data = packs().1.data();
    let mut state = new_state();
    let session = grass_session(&mut state);
    state
        .storage
        .register_capture_in_box(0, mon("TYPHLOSION", 100))
        .unwrap();
    state.sync_party_from_storage();
    let init = data.roaming_pokemon.init_write(0).unwrap();
    let metadata = data.runtime_map_metadata_for_name("Route29").unwrap();
    let roaming = &mut state.roaming_pokemon[0];
    roaming.species = Some(init.species.clone());
    roaming.level = init.level;
    roaming.map_group = metadata.group_id as u8;
    roaming.map_number = metadata.map_id as u8;
    roaming.hp = 0;
    let mut divider = ReplayDivider::new((0..=255).cycle().take(100_000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let roll = (0..1000)
        .find_map(|_| {
            data.check_wild_encounter_after_step(&mut state, &session, &mut rng)
                .unwrap()
                .filter(|roll| roll.roaming_slot == Some(0) && roll.resolved.is_some())
        })
        .expect("fresh roaming chooser");
    let battle = data
        .start_resolved_wild_encounter_after_step(&mut state, &session, &Some(roll), &mut rng)
        .unwrap()
        .unwrap();
    let enemy = &battle.enemy_pokemon;
    assert_eq!(enemy.level, 99);
    assert!(enemy.max_hp > 255);
    assert_eq!(enemy.hp, enemy.max_hp);
    assert_eq!(state.roaming_pokemon[0].current_hp(), enemy.hp);
    let reloaded: GameState = roundtrip_state(&state);
    data.validate_saved_roaming_references(
        &reloaded.roaming_pokemon,
        &reloaded.roaming_map_history,
    )
    .unwrap();
    let engine = RoamingEngineState {
        slots: reloaded.roaming_pokemon,
        history: reloaded.roaming_map_history,
        random_state: rng.state(),
        link_random: None,
    };
    drop(rng);
    let remaining = enemy.hp - 1;
    let (after, _) = battle_end_handle_roam_mons(
        &data.roaming_pokemon,
        &engine,
        RoamingBattleEndInput {
            battle_type: "BATTLETYPE_ROAMING",
            roaming_slot: Some(0),
            enemy_hp: remaining,
            battle_result: 1,
            link_battle: false,
            current_map: crystal_core::systems::special_routines::RoamingMapLocation {
                map_group: metadata.group_id as u8,
                map_number: metadata.map_id as u8,
            },
        },
        &mut divider,
    )
    .unwrap();
    assert_eq!(after.slots[0].current_hp(), remaining);
    let mut rng = CrystalRandom::new(after.random_state, &mut divider);
    let resumed = crystal_core::battle::start::materialize_roaming_wild_battle_with_rng(
        &battle.encounter,
        0,
        &after.slots[0],
        &enemy.species,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
        &mut rng,
    )
    .unwrap();
    assert_eq!(resumed.enemy_pokemon.hp, remaining);
    assert_eq!(resumed.enemy_pokemon.dvs, enemy.dvs);
    drop(rng);
    let mut legacy = engine.clone();
    legacy.slots[0].scaled_hp = None;
    assert!(
        serde_json::to_value(&legacy.slots[0])
            .unwrap()
            .get("scaled_hp")
            .is_none()
    );
    let (stock_after, _) = battle_end_handle_roam_mons(
        &data.roaming_pokemon,
        &legacy,
        RoamingBattleEndInput {
            battle_type: "BATTLETYPE_ROAMING",
            roaming_slot: Some(0),
            enemy_hp: 0x1234,
            battle_result: 1,
            link_battle: false,
            current_map: crystal_core::systems::special_routines::RoamingMapLocation {
                map_group: metadata.group_id as u8,
                map_number: metadata.map_id as u8,
            },
        },
        &mut divider,
    )
    .unwrap();
    assert_eq!(
        stock_after.slots[0].hp, 0x34,
        "stock roamers keep the source low-byte behavior"
    );
    assert_eq!(stock_after.slots[0].scaled_hp, None);
}
