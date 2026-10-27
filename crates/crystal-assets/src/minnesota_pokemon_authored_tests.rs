//! Opt-in validation of actual authored content. Builds in memory; never publishes.
use std::path::PathBuf;

use crystal_core::battle::capture::{CaptureAttemptContext, compute_final_catch_rate};
use crystal_core::battle::turn::{BattleAction, BattleCombatState, BattleTurnInput};
use crystal_core::models::{Dv, LearnedMove, Party, PokedexState};
use crystal_core::random::BattleRandomSource;
use crystal_core::state::{DayCareResidentState, DayCareState};
use crystal_core::systems::evolution::{EvolutionContext, LinkMode, check_and_evolve};
use crystal_core::world::encounters::TimeOfDay;

use super::MinnesotaPokemonCatalog;

struct Random(u32);

impl BattleRandomSource for Random {
    fn battle_random_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 24) as u8
    }
}

fn input_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            panic!("set {name} to the actual authored input path for this opt-in test")
        })
}

#[test]
#[ignore = "requires all reviewed native art, complete authored profiles and a supplied base"]
fn authored_catalog_constructs_parties_battles_evolutions_and_preserves_base() {
    let base = crate::read_verified_compiled_game_pack(input_path("CRYSTAL_RENDER_TEST_PACK"))
        .expect("verified external base");
    let catalog = MinnesotaPokemonCatalog::load(
        &input_path("CRYSTAL_MINNESOTA_POKEMON_ROOT"),
        &input_path("CRYSTAL_MINNESOTA_NATIVE_ROOT"),
        &input_path("CRYSTAL_MINNESOTA_PROFILES"),
    )
    .expect("complete reviewed authored catalog");
    let pack = catalog
        .build(&base, None)
        .expect("full in-memory extension");
    let data = pack.data();
    let added: Vec<_> = data
        .pokemon
        .keys()
        .filter(|id| !base.data().pokemon.contains_key(*id))
        .collect();
    assert_eq!(added.len(), catalog.profiles.len());
    let mut dex = PokedexState::default();
    for id in &added {
        for level in [1, 50, 100] {
            let pokemon = data
                .create_pokemon(id, level, Dv::from_non_hp(10, 10, 10, 10))
                .unwrap_or_else(|error| panic!("construct {id} at {level}: {error}"));
            assert!(pokemon.max_hp > 0 && pokemon.hp == pokemon.max_hp);
            assert!(!pokemon.moves.is_empty() && pokemon.moves.len() <= 4);
            assert!(
                pokemon
                    .moves
                    .iter()
                    .any(|learned| data.moves[&learned.name].power > 0),
                "{id} at level {level} needs a damaging default move"
            );
            for learned in &pokemon.moves {
                assert_eq!(learned.current_pp, data.moves[&learned.name].pp);
            }
            let mut party = Party::default();
            assert!(party.add_pokemon(pokemon.clone()));
            party.validate_saved_state().expect("valid real party");
            data.validate_saved_pokemon_party_references(
                "Minnesota party",
                std::slice::from_ref(&pokemon),
            )
            .expect("party references resolve");
            let day_care = DayCareState {
                man: DayCareResidentState {
                    pokemon: Some(pokemon.clone()),
                    active: true,
                },
                ..DayCareState::default()
            };
            data.validate_saved_day_care_references(&day_care)
                .expect("custom species survives Day Care reference validation");
            compute_final_catch_rate(
                &pokemon,
                &pokemon,
                &CaptureAttemptContext::wild("HEAVY_BALL"),
                &data.capture_rules,
            )
            .expect("explicit Heavy Ball metadata resolves");
            dex.record_caught_pokemon(&pokemon);
        }
        let egg_moves = data.egg_moves[id.as_str()]
            .as_array()
            .expect("explicit egg-move list");
        assert!(egg_moves.iter().all(|value| {
            value
                .as_str()
                .is_some_and(|move_id| data.moves.contains_key(move_id))
        }));
        assert_eq!(data.pokedex_entries[id.as_str()].pages.len(), 2);
    }
    assert_eq!(dex.caught_count(), added.len());
    assert!(
        data.oak_ratings
            .last()
            .expect("Oak coverage")
            .caught_count_limit
            >= data.pokemon.len()
    );

    // Exercise the actual authored learned moves, including the custom number
    // beyond 511 and the new Water/Steel fish, through the authoritative turn engine.
    let highest = added
        .iter()
        .max_by_key(|id| data.pokemon[id.as_str()].int_id)
        .expect("new species");
    let steel_fish = added
        .iter()
        .find(|id| id.as_str() == "MUSKETUNGE")
        .expect("authored Water/Steel fish");
    assert_eq!(data.pokemon[steel_fish.as_str()].type1, "WATER");
    assert_eq!(data.pokemon[steel_fish.as_str()].type2, "STEEL");
    for id in [*highest, *steel_fish] {
        let player = data
            .create_pokemon(id, 1, Dv::default())
            .expect("battle player");
        let slot = player
            .moves
            .iter()
            .position(|learned| data.moves[&learned.name].power > 0)
            .expect("authored damaging level-one move");
        let mut enemy = data
            .create_pokemon("MEW", 20, Dv::default())
            .expect("core opponent");
        enemy.moves = vec![LearnedMove {
            name: "SPLASH".to_string(),
            current_pp: data.moves["SPLASH"].pp,
            pp_ups: 0,
        }];
        let mut combat = BattleCombatState::new(player, enemy);
        let initial_pp = combat.player.moves[slot].current_pp;
        let initial_hp = combat.enemy.hp;
        let mut rng = Random(47);
        for _ in 0..8 {
            combat = data
                .resolve_battle_turn_with_items(
                    combat,
                    BattleTurnInput {
                        player: BattleAction::Move { slot },
                        enemy: BattleAction::Move { slot: 0 },
                    },
                    &mut rng,
                )
                .expect("authoritative custom-species battle turn")
                .state;
            if combat.enemy.hp < initial_hp {
                break;
            }
        }
        assert!(
            combat.player.moves[slot].current_pp < initial_pp,
            "{id} spends real PP"
        );
        assert!(combat.enemy.hp < initial_hp, "{id} deals real damage");
    }

    let mut evolved_count = 0;
    for id in &added {
        for entry in data.evolutions.0[id.as_str()]
            .iter()
            .filter(|entry| entry.method == "LEVEL")
        {
            let mut pokemon = data
                .create_pokemon(
                    id,
                    entry.level.expect("level evolution operand"),
                    Dv::default(),
                )
                .expect("evolution candidate");
            let context = EvolutionContext {
                species: &data.pokemon,
                moves: &data.moves,
                learnsets: &data.learnsets,
                time_of_day: TimeOfDay::Day,
                current_item: None,
                force_evolution: false,
                link_mode: LinkMode::None,
            };
            let report = check_and_evolve(&mut pokemon, &data.evolutions, &context, false)
                .expect("actual authored evolution");
            assert_eq!(
                report.target_species.as_deref(),
                Some(entry.species.as_str())
            );
            assert_eq!(pokemon.species.id, entry.species);
            data.validate_saved_pokemon_reference("evolved custom species", &pokemon)
                .expect("evolved stats and references");
            evolved_count += 1;
        }
    }
    assert!(evolved_count > 0, "real level evolutions exercised");
    assert_eq!(
        pack.compiled_audio, base.compiled_audio,
        "all audio payloads/programs"
    );
    assert_eq!(pack.audio_manifest, base.audio_manifest);
    assert_eq!(pack.audio_compression, base.audio_compression);
    assert_eq!(
        data.audio,
        base.data().audio,
        "authored audio programs unchanged"
    );
    for (key, bytes) in &base.runtime_files {
        assert_eq!(
            pack.runtime_files.get(key),
            Some(bytes),
            "original runtime file {key}"
        );
    }
    for (id, species) in &base.data().pokemon {
        assert_eq!(data.pokemon.get(id), Some(species));
    }
}
