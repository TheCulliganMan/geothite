//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{RuntimeBattleKind, RuntimeShellPhase};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{finish_text, position};
use super::river_valley::fixture;

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
                    "SCRATCH"
                };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == preferred && m.current_pp > 0)
                    .or_else(|| {
                        battle.player_moves.iter().position(|m| {
                            m.current_pp > 0
                                && ["WATER_GUN", "SCRATCH", "THUNDERSHOCK", "TACKLE"]
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

fn use_collected_super_potion(controller: &mut VisibleShellController) {
    finish_text(controller);
    controller
        .press(GameButton::Start)
        .expect("production Start");
    let snapshot = controller.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    for _ in 0..16 {
        let current = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("open PACK");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select PACK");
    }
    let bag = controller.snapshot().expect("carried medicine").bag;
    let target = bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == "SUPER_POTION")
        .expect("collected Super Potion");
    for _ in 0..16 {
        let current = controller
            .shell
            .bag_cursor
            .as_ref()
            .expect("production bag cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("medicine actions");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select medicine");
    }
    let snapshot = controller.snapshot().expect("medicine actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("production item actions");
    let target = actions
        .iter()
        .position(|o| *o == super::super::FieldPackAction::Use)
        .expect("USE action");
    for _ in 0..16 {
        let current = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("use medicine");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select USE");
    }
    assert!(
        controller.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::PartyPokemon),
        "medicine target did not open; selected={:?}, ui={:?}",
        super::super::selected_field_pack_item_id(&mut controller.shell),
        controller
            .snapshot()
            .expect("medicine target diagnostic")
            .ui
    );
    for _ in 0..6 {
        if controller.shell.party_cursor == 0 {
            break;
        }
        controller
            .press(GameButton::Up)
            .expect("select lead partner");
    }
    assert_eq!(controller.shell.party_cursor, 0);
    let before = controller.snapshot().expect("injured Croconaw").party.slots[0]
        .pokemon
        .hp;
    controller
        .press(GameButton::A)
        .expect("apply Super Potion to Croconaw");
    let healed = controller.snapshot().expect("native healing result");
    assert!(
        healed.party.slots[0].pokemon.hp > before,
        "medicine did not restore HP: before={before}, after={}, target={:?}, notice={:?}, field_notice={:?}, ui={:?}",
        healed.party.slots[0].pokemon.hp,
        controller.shell.field_pack_target_mode,
        controller.shell.visible_field_item_notice,
        controller.shell.field_notice,
        healed.ui
    );
    assert!(
        !healed
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "SUPER_POTION" && s.quantity > 0),
        "medicine not consumed"
    );
    for _ in 0..16 {
        let current = controller.snapshot().expect("close field Pack");
        if controller.shell.field_notice.is_some() || current.ui.pending_text_wait.is_some() {
            controller
                .wait_frames(60)
                .expect("healing presentation frames");
            controller.press(GameButton::A).expect("healing notice");
        } else if current.ui.menu.is_some() {
            controller
                .press(GameButton::B)
                .expect("close medicine menu");
        } else {
            return;
        }
    }
    let final_ui = controller.snapshot().expect("healing close diagnostic").ui;
    panic!(
        "medicine did not return to walking: field_notice={:?}, item_notice={:?}, ui={:?}",
        controller.shell.field_notice, controller.shell.visible_field_item_notice, final_ui
    );
}

#[test]
#[ignore = "requires the external River Valley pack"]
fn river_valley_journey_crosses_both_bridges_habitat_and_all_trainers_with_saved_pp() {
    let (root, runtime, mut controller) = fixture();
    // Normal source-created partners after the second badge. This placement,
    // party, badge setup and replay clock do not prove earned Q02 progression.
    for (species, level) in [("CROCONAW", 18), ("FLAAFFY", 16)] {
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
    state.badges.johto = [true, true, false, false, false, false, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(vec![0; 131072]);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut journey = Journey::default();
    for p in [
        (18, 56),
        (24, 56),
        (24, 54),
        (39, 54),
        (39, 42),
        (30, 42),
        (30, 28),
        (14, 28),
        (14, 16),
        (15, 16),
    ] {
        journey.walk(&mut controller, p);
    }
    assert!(
        journey.wild_encounters > 0,
        "habitat produced no native encounter"
    );
    controller.press(GameButton::Down).expect("face Jo");
    controller.press(GameButton::A).expect("challenge Jo");
    journey.settle(&mut controller);
    for p in [(14, 16), (14, 14), (20, 14), (20, 2), (20, 4), (21, 4)] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Right, "MINNEAPOLIS");
    for p in [
        (20, 4),
        (20, 14),
        (14, 14),
        (14, 28),
        (30, 28),
        (30, 42),
        (41, 42),
        (41, 39),
        (39, 39),
        (39, 36),
    ] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Up, "found");
    journey.walk(&mut controller, (37, 36));
    controller.press(GameButton::Down).expect("face Ed");
    controller.press(GameButton::A).expect("challenge Ed");
    journey.settle(&mut controller);
    for p in [(39, 36), (39, 39), (37, 39), (37, 42)] {
        journey.walk(&mut controller, p);
    }
    for _ in 0..4 {
        controller
            .press(GameButton::Up)
            .expect("try terrace reverse");
    }
    assert_eq!(position(&mut controller), (37, 42));
    for p in [(38, 42), (38, 48), (37, 48)] {
        journey.walk(&mut controller, p);
    }
    controller.press(GameButton::Down).expect("face Cal");
    controller.press(GameButton::A).expect("challenge Cal");
    journey.settle(&mut controller);
    for p in [(39, 48), (39, 54), (39, 55), (40, 55)] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Right, "found");
    use_collected_super_potion(&mut controller);
    for p in [(39, 55), (39, 54), (24, 54), (24, 56), (18, 56), (18, 58)] {
        journey.walk(&mut controller, p);
    }
    assert_eq!(journey.attacks_spent_pp, 4);
    for species in [
        "WOOPER",
        "POLIWAG",
        "HOPPIP",
        "PIDGEY",
        "BELLSPROUT",
        "MAREEP",
        "SLOWPOKE",
    ] {
        assert!(
            journey.trainer_opponents.contains(species),
            "missing {species}"
        );
    }
    let flags = [
        "EVENT_MN_R03_TRAINER_01",
        "EVENT_MN_R03_TRAINER_02",
        "EVENT_MN_R03_TRAINER_03",
        "EVENT_MN_R03_TRAINER_04",
        "EVENT_MN_R03_ITEM_04",
        "EVENT_MN_R03_ITEM_01",
    ];
    for f in flags {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(f)
                .expect("journey flag"),
            "{f}"
        );
    }
    let before = controller.snapshot().expect("journey party");
    assert!(
        before
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "ETHER" && s.quantity == 1)
    );
    let save = std::env::temp_dir().join(format!(
        "geothite-river-valley-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save journey");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    assert_eq!(position(&mut loaded), (18, 58));
    let after = loaded.snapshot().expect("continued journey");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    for f in flags {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(f)
                .expect("continued flag")
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_RIVER_VALLEY_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export native journey");
    }
    std::fs::remove_file(save).expect("remove scratch save");
    eprintln!(
        "River Valley: {} wild encounters, both bridge crossings, four trainers/seven opponents defeated with spent PP, Ether, used Super Potion and party/flags survive Continue",
        journey.wild_encounters
    );
}
