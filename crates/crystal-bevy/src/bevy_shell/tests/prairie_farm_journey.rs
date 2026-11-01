//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{finish_text, position};

type Point = (i16, i16);

#[derive(Default)]
struct Journey {
    wild_encounters: usize,
    trainer_opponents: BTreeSet<String>,
    attacks_spent_pp: usize,
}

impl Journey {
    fn settle(&mut self, controller: &mut VisibleShellController) {
        for _ in 0..64 {
            let s = controller.snapshot().expect("visible activity");
            if s.battle.is_some() {
                self.battle(controller);
            } else if s.phase == RuntimeShellPhase::Text || controller.has_pending_script_work() {
                controller.press(GameButton::A).expect("native text page");
            } else {
                return;
            }
        }
        panic!("route activity failed to settle");
    }

    fn battle(&mut self, controller: &mut VisibleShellController) {
        let initial = controller.snapshot().expect("encounter");
        let wild = matches!(
            initial.battle.as_ref().expect("battle").kind,
            RuntimeBattleKind::Wild { .. }
        );
        self.wild_encounters += usize::from(wild);
        let initial_pp: u32 = initial
            .party
            .slots
            .iter()
            .flat_map(|p| &p.pokemon.moves)
            .map(|m| u32::from(m.current_pp))
            .sum();
        for _ in 0..768 {
            let s = controller.snapshot().expect("production battle");
            let Some(battle) = &s.battle else {
                break;
            };
            if !wild {
                self.trainer_opponents
                    .insert(battle.enemy_pokemon.species.id.clone());
            }
            let options =
                s.ui.menu
                    .as_ref()
                    .map(|menu| {
                        menu.layout
                            .vertical_menus
                            .iter()
                            .flat_map(|m| &m.options)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            let button = if !controller.shell.battle_messages.is_empty()
                || (s.ui.menu.is_none() && s.ui.text.is_some())
            {
                GameButton::A
            } else if options.iter().any(|o| o.contains("change <PKMN>")) {
                GameButton::B
            } else if options.iter().any(|o| o.contains("FIGHT")) {
                let target = if wild { 3 } else { 0 };
                let cursor = controller
                    .shell
                    .battle_action_cursor
                    .as_ref()
                    .expect("action cursor")
                    .option_index;
                if cursor / 2 < target / 2 {
                    GameButton::Down
                } else if cursor / 2 > target / 2 {
                    GameButton::Up
                } else if cursor % 2 < target % 2 {
                    GameButton::Right
                } else if cursor % 2 > target % 2 {
                    GameButton::Left
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_party_action_cursor {
                if cursor.option_index > 0 {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_switch_cursor {
                let target = s
                    .party
                    .slots
                    .iter()
                    .position(|p| {
                        p.pokemon.hp > 0 && Some(p.index) != battle.active_player_party_index
                    })
                    .expect("living replacement");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_move_cursor {
                let slot = battle.active_player_party_index.expect("active partner");
                let pokemon = &s.party.slots[slot].pokemon;
                let preferred = if pokemon.species.id == "ZAPSAP" {
                    "THUNDERSHOCK"
                } else {
                    "EMBER"
                };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == preferred && m.current_pp > 0)
                    .or_else(|| {
                        battle.player_moves.iter().position(|m| {
                            m.current_pp > 0
                                && ["EMBER", "THUNDERSHOCK", "TACKLE"].contains(&m.name.as_str())
                        })
                    })
                    .expect("usable legal move");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else {
                GameButton::A
            };
            controller.press(button).expect("production battle input");
            controller
                .wait_frames(16)
                .expect("native battle narration presentation");
        }
        let ended = controller.snapshot().expect("battle result");
        assert!(ended.battle.is_none(), "battle did not end: {:?}", ended.ui);
        if !wild {
            let final_pp: u32 = ended
                .party
                .slots
                .iter()
                .flat_map(|p| &p.pokemon.moves)
                .map(|m| u32::from(m.current_pp))
                .sum();
            assert!(final_pp < initial_pp, "trainer battle never spent move PP");
            self.attacks_spent_pp += 1;
        }
        finish_text(controller);
    }

    fn read(
        &mut self,
        controller: &mut VisibleShellController,
        facing: GameButton,
        expected: &str,
    ) {
        controller.press(facing).expect("face native interaction");
        controller.press(GameButton::A).expect("read or speak");
        let snapshot = controller.snapshot().expect("visible directions");
        assert!(
            snapshot
                .ui
                .text
                .as_ref()
                .and_then(|t| t.asm_text.as_ref())
                .is_some_and(|text| text.contains(expected)),
            "missing {expected}: {:?}",
            snapshot.ui
        );
        let tile = snapshot.overworld.tile;
        controller
            .press(GameButton::Down)
            .expect("movement while text owns input");
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue input owner")
                .overworld
                .tile,
            tile
        );
        self.settle(controller);
    }

    fn walk(&mut self, controller: &mut VisibleShellController, target: Point) {
        let start = position(controller);
        assert!(
            start.0 == target.0 || start.1 == target.1,
            "hand-authored cardinal segment"
        );
        let button = if target.0 > start.0 {
            GameButton::Right
        } else if target.0 < start.0 {
            GameButton::Left
        } else if target.1 > start.1 {
            GameButton::Down
        } else {
            GameButton::Up
        };
        for _ in 0..256 {
            if position(controller) == target {
                return;
            }
            controller.press(button).expect("route movement");
            self.settle(controller);
        }
        panic!(
            "blocked hand-authored segment {start:?} -> {target:?}, at {:?}",
            position(controller)
        );
    }
}

#[test]
#[ignore = "requires the external compact Prairie Farm pack"]
fn prairie_farm_journey_crosses_grass_bank_bridge_and_wins_all_three_with_saved_pp() {
    let pack =
        PathBuf::from(std::env::var_os("CRYSTAL_PRAIRIE_FARM_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack).expect("native pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR02C00R00".into(),
            tile_x: 54,
            tile_y: 14,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit first-badge placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    // Source-created levels and learned moves, one badge and a deterministic
    // divider are explicit setup. All subsequent traversal/battle uses input.
    for (species, level) in [("KINDLIT", 12), ("ZAPSAP", 9)] {
        let pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("source-legal first-badge partner");
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(pokemon)
        );
    }
    let state = controller.shell.shell.session_mut().state_mut();
    state.badges.johto = [true, false, false, false, false, false, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut journey = Journey::default();
    journey.walk(&mut controller, (56, 14));
    journey.read(&mut controller, GameButton::Up, "PRAIRIE FARM");
    for p in [(48, 14), (48, 15), (45, 15)] {
        journey.walk(&mut controller, p);
    }
    for _ in 0..4 {
        controller
            .press(GameButton::Right)
            .expect("try bank reverse");
    }
    assert_eq!(position(&mut controller), (45, 15));
    for p in [
        (40, 15),
        (40, 24),
        (26, 24),
        (20, 24),
        (20, 30),
        (2, 30),
        (20, 30),
        (20, 24),
        (40, 24),
        (40, 26),
        (52, 26),
        (52, 21),
    ] {
        journey.walk(&mut controller, p);
    }
    assert!(
        journey.wild_encounters > 0,
        "native grass never triggered an encounter"
    );
    controller.press(GameButton::Left).expect("face Tess");
    controller.press(GameButton::A).expect("challenge Tess");
    journey.settle(&mut controller);
    // Visit both small farm spurs as ordinary walking/battle decisions too.
    for p in [(52, 26), (40, 26), (40, 24), (20, 24), (20, 20), (23, 20)] {
        journey.walk(&mut controller, p);
    }
    controller.press(GameButton::Up).expect("face Drew");
    controller.press(GameButton::A).expect("challenge Drew");
    journey.settle(&mut controller);
    for p in [
        (20, 20),
        (20, 24),
        (40, 24),
        (40, 26),
        (48, 26),
        (48, 35),
        (47, 35),
        (47, 36),
    ] {
        journey.walk(&mut controller, p);
    }
    controller.press(GameButton::Down).expect("face Nell");
    controller.press(GameButton::A).expect("challenge Nell");
    journey.settle(&mut controller);
    assert_eq!(journey.attacks_spent_pp, 3);
    assert_eq!(
        journey.trainer_opponents,
        BTreeSet::from([
            "PODLET".into(),
            "PINIT".into(),
            "ZAPSAP".into(),
            "PECKIT".into(),
            "NIBBIN".into()
        ])
    );
    let flags = [
        "EVENT_MN_R02_TRAINER_01",
        "EVENT_MN_R02_TRAINER_02",
        "EVENT_MN_R02_TRAINER_03",
    ];
    for flag in flags {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("trainer defeated")
        );
    }
    for p in [(47, 35), (48, 35), (48, 28), (52, 28), (52, 14), (54, 14)] {
        journey.walk(&mut controller, p);
    }
    let before = controller.snapshot().expect("post-battle party");
    assert!(before.party.slots.iter().any(|p| p.pokemon.hp > 0));
    let save = std::env::temp_dir().join(format!(
        "geothite-prairie-farm-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save journey");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    assert_eq!(position(&mut loaded), (54, 14));
    let after = loaded.snapshot().expect("continued party");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    for flag in flags {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("saved trainer flag")
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_PRAIRIE_FARM_JOURNEY_SAVE") {
        loaded.save(PathBuf::from(path)).expect("export journey");
    }
    std::fs::remove_file(save).expect("remove scratch save");
    eprintln!(
        "Prairie Farm journey: {} wild encounters escaped; all three trainers and five partners defeated through native input; PP, party and flag survive Continue",
        journey.wild_encounters
    );
}
