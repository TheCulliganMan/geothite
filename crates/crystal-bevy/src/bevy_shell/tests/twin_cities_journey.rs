//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{RuntimeBattleKind, RuntimeShellPhase};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{finish_text, position};
use super::twin_cities::fixture;

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
                let enemy = battle.enemy_pokemon.species.id.as_str();
                let preferred = if pokemon.species.id == "FLAAFFY" {
                    if enemy == "WOOPER" {
                        "TACKLE"
                    } else {
                        "THUNDERSHOCK"
                    }
                } else if ["WOOPER", "MAREEP", "PIDGEY"].contains(&enemy) {
                    "WATER_GUN"
                } else {
                    "BITE"
                };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == preferred && m.current_pp > 0)
                    .or_else(|| {
                        battle.player_moves.iter().position(|m| {
                            m.current_pp > 0
                                && ["WATER_GUN", "BITE", "THUNDERSHOCK", "TACKLE"]
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

fn select_cursor(
    controller: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
    forward: GameButton,
    backward: GameButton,
) {
    for _ in 0..16 {
        let current = index(controller);
        if current == target {
            return;
        }
        controller
            .press(if current < target { forward } else { backward })
            .expect("production cursor input");
    }
    panic!("cursor did not reach {target}");
}

fn use_collected_ether(
    controller: &mut VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
) {
    finish_text(controller);
    let before = controller.snapshot().expect("spent battle PP");
    let lead = &before.party.slots[0].pokemon;
    let move_index = lead
        .moves
        .iter()
        .position(|m| m.current_pp < runtime.data().moves[&m.name].pp)
        .expect("actually spent move");
    let move_id = lead.moves[move_index].name.clone();
    let original_pp = lead.moves[move_index].current_pp;
    let amount = runtime.data().items["ETHER"]
        .pp_restore_points
        .expect("source PP amount");
    let expected_pp = original_pp
        .saturating_add(amount)
        .min(runtime.data().moves[&move_id].pp);
    controller
        .press(GameButton::Start)
        .expect("production Start");
    let snapshot = controller.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK");
    select_cursor(
        controller,
        target,
        |c| {
            c.shell
                .start_menu_cursor
                .as_ref()
                .expect("Start cursor")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    controller.press(GameButton::A).expect("open Pack");
    let target = controller
        .snapshot()
        .expect("bag")
        .bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == "ETHER")
        .expect("collected Ether");
    select_cursor(
        controller,
        target,
        |c| {
            c.shell
                .bag_cursor
                .as_ref()
                .expect("bag cursor")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    controller.press(GameButton::A).expect("Ether actions");
    let snapshot = controller.snapshot().expect("actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("logical actions");
    let target = actions
        .iter()
        .position(|o| *o == super::super::FieldPackAction::Use)
        .expect("USE");
    select_cursor(
        controller,
        target,
        |c| {
            c.shell
                .field_pack_action_cursor
                .as_ref()
                .expect("item actions")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    controller.press(GameButton::A).expect("use Ether");
    assert!(
        controller.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::PartyMove),
        "Ether lost its move-target capability"
    );
    select_cursor(
        controller,
        0,
        |c| c.shell.party_cursor,
        GameButton::Down,
        GameButton::Up,
    );
    select_cursor(
        controller,
        move_index,
        |c| {
            c.shell
                .party_move_cursor
                .as_ref()
                .expect("PP move cursor")
                .option_index
        },
        GameButton::Right,
        GameButton::Left,
    );
    controller
        .press(GameButton::A)
        .expect("restore selected move PP");
    let after = controller.snapshot().expect("native PP restoration");
    assert_eq!(
        after.party.slots[0].pokemon.moves[move_index].current_pp,
        expected_pp
    );
    assert!(expected_pp > original_pp);
    assert_eq!(
        after.party.slots[0].pokemon.hp, lead.hp,
        "PP item altered HP"
    );
    assert!(
        !after
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "ETHER" && s.quantity > 0)
    );
    for _ in 0..16 {
        let s = controller.snapshot().expect("close Pack");
        if controller.shell.field_notice.is_some() || s.ui.pending_text_wait.is_some() {
            controller.wait_frames(60).expect("PP notice frames");
            controller.press(GameButton::A).expect("PP notice");
        } else if s.ui.menu.is_some() {
            controller.press(GameButton::B).expect("close menu");
        } else {
            return;
        }
    }
    panic!("Ether did not release input");
}

#[test]
#[ignore = "requires the external Twin Cities pack"]
fn twin_cities_journey_bypasses_cole_both_ways_wins_trainers_uses_ether_and_continues() {
    let (root, runtime, mut controller) = fixture();
    // Source-created partners, three badge bits, placement and the divider are
    // explicit local setup. They do not represent an earned regional journey.
    for (species, level) in [("CROCONAW", 21), ("FLAAFFY", 20)] {
        let pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("source-legal partner");
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
    state.badges.johto = [true, true, true, false, false, false, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(vec![0; 131072]);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut journey = Journey::default();
    for at in [(12, 18), (12, 13), (8, 13)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Left, "found");
    for at in [(12, 13), (12, 9)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Up, "ROSE WALK");
    for at in [(12, 10), (23, 10), (23, 18), (52, 18), (52, 17)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Up, "ST PAUL");
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R04_TRAINER_01")
            .expect("Cole bypass flag")
    );
    assert!(
        journey.wild_encounters > 0,
        "grass alternative never encountered a wild partner"
    );
    for at in [(52, 18), (40, 18), (40, 27), (48, 27)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Right, "found");
    for at in [(40, 27), (40, 25)] {
        journey.walk(&mut controller, at);
    }
    controller.press(GameButton::Right).expect("face Mica");
    controller.press(GameButton::A).expect("challenge Mica");
    journey.settle(&mut controller);
    for at in [(40, 26), (42, 26), (42, 25)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Up, "RIVER PARK");
    for at in [(42, 26), (40, 26), (40, 25), (40, 18), (36, 18), (36, 15)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Up, "MISSISSIPPI");
    for at in [(36, 18), (22, 18), (22, 23)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Right, "stone banks");
    // Return via the habitat, still without triggering Cole's short lane.
    for at in [
        (22, 18),
        (23, 18),
        (23, 10),
        (12, 10),
        (12, 18),
        (10, 18),
        (10, 26),
        (7, 26),
    ] {
        journey.walk(&mut controller, at);
    }
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R04_TRAINER_01")
            .expect("reverse Cole bypass")
    );
    journey.read(&mut controller, GameButton::Up, "flowers");
    journey.walk(&mut controller, (9, 26));
    journey.read(&mut controller, GameButton::Down, "found");
    for at in [(10, 26), (10, 18), (16, 18), (18, 18)] {
        journey.walk(&mut controller, at);
    }
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R04_TRAINER_01")
            .expect("native sight battle won")
    );
    // Cole remains adjacent to the pinch. His body leaves the lower passing
    // bay clear after defeat; prove the short lane in both directions.
    for at in [
        (18, 19),
        (20, 19),
        (20, 18),
        (52, 18),
        (20, 18),
        (20, 19),
        (18, 19),
        (18, 18),
        (12, 18),
    ] {
        journey.walk(&mut controller, at);
    }
    use_collected_ether(&mut controller, &runtime);
    for at in [(4, 18), (4, 17), (2, 17)] {
        journey.walk(&mut controller, at);
    }
    journey.read(&mut controller, GameButton::Up, "MINNEAPOLIS");
    for at in [(4, 17), (4, 18)] {
        journey.walk(&mut controller, at);
    }
    assert_eq!(journey.attacks_spent_pp, 2);
    for species in ["DITTO", "YANMA", "ABRA", "SNUBBULL"] {
        assert!(journey.trainer_opponents.contains(species), "{species}");
    }
    let flags = [
        "EVENT_MN_R04_TRAINER_01",
        "EVENT_MN_R04_TRAINER_02",
        "EVENT_MN_R04_ITEM_01",
        "EVENT_MN_R04_ITEM_02",
        "EVENT_MN_R04_ITEM_04",
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
                .expect("journey flag")
        );
    }
    let before = controller.snapshot().expect("final party and inventory");
    let save = std::env::temp_dir().join(format!(
        "geothite-twin-cities-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save journey");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    assert_eq!(position(&mut loaded), (4, 18));
    let after = loaded.snapshot().expect("continued party and inventory");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    for flag in flags {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("continued flag")
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_TWIN_CITIES_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export journey save");
    }
    std::fs::remove_file(save).expect("remove temporary save");
    eprintln!(
        "Twin Cities: {} wild encounters, both grass bypass directions, two trainer victories/four opponents with spent PP, native Ether restoration and saved party/bag/flags",
        journey.wild_encounters
    );
}
