use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::battle::{start::LinkBattleStart, turn::BattleAction};
use crystal_core::models::pokemon::Dv;
use crystal_core::random::{CrystalRandomState, LinkBattleRandomState, ReplayDivider};
use crystal_core::state::LinkSerialConnectionStatus;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

#[test]
#[ignore = "requires the external Crystal pack"]
fn mirrored_link_turns_use_shared_rng_and_reach_opposite_results() {
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack");
    let root = AssetRoot::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(std::path::Path::new(&pack)).unwrap(),
    )
    .unwrap();
    for species in ["SQUIRTLE", "CHARMANDER"] {
        let mut shells = Vec::new();
        for (index, mon) in ["CHARMANDER", species].into_iter().enumerate() {
            let mut shell = RuntimeGameShell::new_game(
                root.clone(),
                runtime.clone(),
                runtime.title_new_game_spawn_identifier().unwrap(),
            )
            .unwrap();
            shell
                .add_party_pokemon(
                    mon,
                    5,
                    None,
                    None,
                    "TRAINER",
                    index as u16 + 1,
                    Dv::from_non_hp(10, 10, 10, 10),
                )
                .unwrap();
            let state = shell.session_mut().state_mut();
            state.link_session.link_mode = 3;
            state.link_session.serial_connection_status = if index == 0 {
                LinkSerialConnectionStatus::UsingInternalClock
            } else {
                LinkSerialConnectionStatus::UsingExternalClock
            };
            state.link_session.battle_random = Some(LinkBattleRandomState {
                seeds: [31, 127, 223, 54, 118, 77, 159, 229, 86, 213],
                count: 0,
            });
            state.random_state = CrystalRandomState {
                add: index as u8 * 101,
                sub: 37,
            };
            shells.push(shell);
        }
        let parties: Vec<_> = shells
            .iter()
            .map(|shell| {
                shell
                    .session()
                    .state()
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        for (index, shell) in shells.iter_mut().enumerate() {
            shell
                .start_link_battle(&LinkBattleStart {
                    opponent_player_id: 2 - index as u64,
                    opponent_name: "PEER".into(),
                    enemy_party: parties[1 - index].clone(),
                    random_state: shell
                        .session()
                        .state()
                        .link_session
                        .battle_random
                        .clone()
                        .unwrap(),
                })
                .unwrap();
        }
        let mut states: Vec<_> = shells
            .iter()
            .map(|shell| shell.session().state().clone())
            .collect();
        let initial_rng: Vec<_> = states.iter().map(|state| state.random_state).collect();
        let mut terminal = false;
        for _ in 0..100 {
            let outcomes: Vec<_> = states
                .iter_mut()
                .map(|state| {
                    runtime
                        .data()
                        .resolve_active_battle_turn_with_divider(
                            state,
                            BattleAction::Move { slot: 0 },
                            BattleAction::Move { slot: 0 },
                            &mut ReplayDivider::new([]),
                        )
                        .expect("link turns must never read the local DIV clock")
                })
                .collect();
            assert_eq!(outcomes[0].state.player.hp, outcomes[1].state.enemy.hp);
            assert_eq!(outcomes[0].state.enemy.hp, outcomes[1].state.player.hp);
            assert_eq!(
                outcomes[0].state.player.moves,
                outcomes[1].state.enemy.moves
            );
            assert_eq!(
                outcomes[0].state.enemy.moves,
                outcomes[1].state.player.moves
            );
            assert_eq!(
                states[0].link_session.battle_random,
                states[1].link_session.battle_random
            );
            assert_eq!(states[0].random_state, initial_rng[0]);
            assert_eq!(states[1].random_state, initial_rng[1]);
            let outcome = &outcomes[0].state;
            if outcome.player.hp == 0 || outcome.enemy.hp == 0 {
                assert_ne!(outcome.player.hp == 0, outcome.enemy.hp == 0);
                terminal = true;
                break;
            }
        }
        assert!(
            terminal,
            "both perspectives must finish, including equal-speed parties"
        );
    }
}
