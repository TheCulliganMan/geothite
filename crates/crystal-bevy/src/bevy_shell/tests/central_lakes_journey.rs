//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{finish_text, indoor_walk, position};

type Point = (i16, i16);

#[derive(Default)]
struct Journey {
    wild_encounters: usize,
    trainer_opponents: BTreeSet<String>,
    attacks_spent_pp: usize,
    battle_clock_started: bool,
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
        if !self.battle_clock_started {
            // The first grass roll is deterministic. Vary subsequent divider
            // samples so normal battle rejection sampling can terminate.
            *controller.shell.shell.session_mut().divider_mut_for_tests() =
                crystal_core::random::RuntimeDividerSource::replay(
                    (0_u32..131072)
                        .map(|n| ((n * 73 + 19) % 256) as u8)
                        .collect::<Vec<u8>>(),
                );
            self.battle_clock_started = true;
        }
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
            let button = if options.iter().any(|o| o.contains("change <PKMN>")) {
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
                let preferred =
                    if pokemon.hp * 3 < pokemon.max_hp && pokemon.species.id == "MEGANIUM" {
                        "SYNTHESIS"
                    } else if pokemon.species.id == "LAPRAS" {
                        "ICE_BEAM"
                    } else if ["POLIWHIRL", "GOLDEEN"]
                        .contains(&battle.enemy_pokemon.species.id.as_str())
                    {
                        "RAZOR_LEAF"
                    } else {
                        "BODY_SLAM"
                    };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == preferred && m.current_pp > 0)
                    .or_else(|| {
                        battle.player_moves.iter().position(|m| {
                            m.current_pp > 0
                                && ["RAZOR_LEAF", "BODY_SLAM", "ICE_BEAM", "WATER_GUN"]
                                    .contains(&m.name.as_str())
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
#[ignore = "requires the external compact Central Lakes pack"]
fn central_lakes_journey_uses_grass_ledge_bypass_real_trainers_and_saved_flags() {
    let pack =
        PathBuf::from(std::env::var_os("CRYSTAL_CENTRAL_LAKES_TEST_PACK").expect("external pack"));
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
            map_name: "MnR15C00R00".into(),
            tile_x: 4,
            tile_y: 24,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit western placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    // Normal mid-game levels, source-legal retained moves and six badge bits
    // are explicit test setup. Nothing here claims earned regional progression.
    for (species, level, moves) in [
        (
            "MEGANIUM",
            34,
            vec!["RAZOR_LEAF", "BODY_SLAM", "SYNTHESIS", "REFLECT"],
        ),
        (
            "LAPRAS",
            36,
            vec!["ICE_BEAM", "BODY_SLAM", "WATER_GUN", "CONFUSE_RAY"],
        ),
    ] {
        let mut pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal partner");
        pokemon.moves = moves
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
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
    state.badges.johto = [true, true, true, true, true, true, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(vec![0; 131072]);
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut journey = Journey::default();
    for target in [(3, 24), (3, 22)] {
        journey.walk(&mut controller, target);
    }
    journey.read(&mut controller, GameButton::Up, "CENTRAL LAKES");
    for target in [(3, 24), (4, 24)] {
        journey.walk(&mut controller, target);
    }
    // The shared grass gate and longer northern meadow are actual encounters.
    for target in [(14, 24), (14, 12), (22, 12), (22, 9)] {
        journey.walk(&mut controller, target);
    }
    controller
        .press(GameButton::Right)
        .expect("face meadow reward");
    controller
        .press(GameButton::A)
        .expect("collect meadow reward");
    journey.settle(&mut controller);
    journey.walk(&mut controller, (22, 20)); // includes the native downhill hop
    for _ in 0..4 {
        controller.press(GameButton::Up).expect("try reverse ledge");
    }
    assert_eq!(position(&mut controller), (22, 20));
    for target in [(22, 24), (22, 28), (46, 28), (46, 26), (49, 26)] {
        journey.walk(&mut controller, target);
    }
    journey.read(&mut controller, GameButton::Up, "SHINER LAKE");
    for target in [(46, 26), (46, 17)] {
        journey.walk(&mut controller, target);
    }
    journey.read(&mut controller, GameButton::Left, "rail ends");
    for target in [(46, 15), (38, 15)] {
        journey.walk(&mut controller, target);
    }
    controller
        .press(GameButton::Left)
        .expect("face dock reward");
    controller
        .press(GameButton::A)
        .expect("collect dock reward");
    journey.settle(&mut controller);
    // Walk the attached woodland S bend through its open southern lip.
    for target in [
        (46, 15),
        (46, 21),
        (50, 21),
        (50, 20),
        (54, 20),
        (54, 24),
        (60, 24),
    ] {
        journey.walk(&mut controller, target);
    }
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R15_TRAINER_02")
            .expect("Ron flag"),
        "the grass/ledge route failed to bypass Ron"
    );
    assert!(
        journey.wild_encounters > 0,
        "grass never produced a real encounter"
    );
    // Optional campground trainer, challenged by A rather than a state change.
    for target in [(54, 24), (54, 18), (55, 18)] {
        journey.walk(&mut controller, target);
    }
    controller.press(GameButton::Up).expect("face Niko");
    controller.press(GameButton::A).expect("challenge Niko");
    journey.settle(&mut controller);
    // Actual interior reward, with the already-verified safe floor return.
    for target in [(54, 18), (54, 20), (57, 20)] {
        journey.walk(&mut controller, target);
    }
    journey.read(&mut controller, GameButton::Up, "LAKE CABIN");
    for target in [(54, 20), (54, 12), (56, 12), (56, 13)] {
        journey.walk(&mut controller, target);
    }
    journey.read(&mut controller, GameButton::Right, "Eli lives here");
    for target in [(56, 12), (53, 12)] {
        journey.walk(&mut controller, target);
    }
    controller.press(GameButton::Up).expect("face door");
    controller.press(GameButton::Up).expect("enter cabin");
    assert_eq!(
        controller.snapshot().expect("cabin").overworld.map_name,
        "MnR15LakeCabin"
    );
    indoor_walk(&mut controller, &runtime, (6, 3));
    journey.read(&mut controller, GameButton::Right, "Camp Lake changes");
    indoor_walk(&mut controller, &runtime, (4, 1));
    journey.read(&mut controller, GameButton::Up, "CHAIN OF LAKES");
    indoor_walk(&mut controller, &runtime, (7, 6));
    controller.press(GameButton::Up).expect("face cabin cache");
    controller
        .press(GameButton::A)
        .expect("collect cabin cache");
    journey.settle(&mut controller);
    indoor_walk(&mut controller, &runtime, (5, 7));
    for _ in 0..4 {
        if controller
            .snapshot()
            .expect("room departure")
            .overworld
            .map_name
            != "MnR15LakeCabin"
        {
            break;
        }
        controller.press(GameButton::Down).expect("leave carpet");
    }
    assert_eq!(position(&mut controller), (53, 12));
    // Return west: the one-way detour cannot be climbed, so Ron controls the lane.
    for target in [
        (53, 14),
        (54, 14),
        (54, 20),
        (50, 20),
        (50, 21),
        (46, 21),
        (46, 28),
        (21, 28),
        (21, 25),
        (14, 25),
        (4, 25),
    ] {
        journey.walk(&mut controller, target);
    }
    // Ron was skipped eastbound and defeated on the return. His persistent
    // flag now permits the ordinary uphill walking loop, never a reverse hop.
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R15_TRAINER_02")
            .expect("Ron defeated")
    );
    for target in [(14, 25), (14, 12), (22, 12), (22, 17), (22, 20)] {
        journey.walk(&mut controller, target);
    }
    for _ in 0..4 {
        controller
            .press(GameButton::Up)
            .expect("reject reverse after victory");
    }
    assert_eq!(position(&mut controller), (22, 20));
    // Exact 31-tile return, through the western entrance and Ron's old sight.
    for target in [(22, 25), (15, 25), (15, 15), (22, 15), (22, 17)] {
        journey.walk(&mut controller, target);
    }
    for target in [(22, 20), (22, 25), (4, 25)] {
        journey.walk(&mut controller, target);
    }
    assert!(journey.attacks_spent_pp >= 2);
    for species in ["NIDORINO", "PINECO", "POLIWHIRL", "GOLDEEN"] {
        assert!(
            journey.trainer_opponents.contains(species),
            "missing trainer opponent {species}"
        );
    }
    for flag in [
        "EVENT_MN_R15_TRAINER_02",
        "EVENT_MN_R15_TRAINER_03",
        "EVENT_MN_R15_ITEM_01",
        "EVENT_MN_R15_ITEM_04",
        "EVENT_MN_R15_CABIN_ETHER",
    ] {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("journey flag"),
            "{flag}"
        );
    }
    let save = std::env::temp_dir().join(format!(
        "geothite-central-lakes-journey-{}.crystalsave",
        std::process::id()
    ));
    let before_save = controller.snapshot().expect("journey inventory and party");
    for item in ["HYPER_POTION", "REVIVE", "MAX_ETHER"] {
        assert!(
            before_save
                .bag
                .items
                .iter()
                .any(|s| s.item_id == item && s.quantity == 1),
            "missing inventory {item}"
        );
    }
    controller.save(&save).expect("save journey");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    assert_eq!(position(&mut loaded), (4, 25));
    let after_save = loaded.snapshot().expect("continued party and inventory");
    assert_eq!(after_save.party, before_save.party);
    assert_eq!(after_save.bag, before_save.bag);
    for flag in [
        "EVENT_MN_R15_TRAINER_02",
        "EVENT_MN_R15_TRAINER_03",
        "EVENT_MN_R15_ITEM_01",
        "EVENT_MN_R15_ITEM_04",
        "EVENT_MN_R15_CABIN_ETHER",
    ] {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("saved flag"),
            "{flag}"
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_CENTRAL_LAKES_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export explicit journey fixture");
    }
    std::fs::remove_file(save).expect("remove temporary save");
    eprintln!(
        "Central Lakes input journey: {} wild encounters escaped, {} trainer victories with spent PP, all four trainer species observed; party, bag and flags survive Continue",
        journey.wild_encounters, journey.attacks_spent_pp
    );
}
