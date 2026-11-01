//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{
    RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, opposite, position,
};
use super::river_bluffs::fixture;

type Point = (i16, i16);

struct Journey<'a> {
    runtime: &'a CrystalRuntime,
    wild_encounters: usize,
    trainer_opponents: BTreeSet<String>,
    attacks_spent_pp: usize,
    medicine_uses: usize,
}

impl Journey<'_> {
    fn settle(&mut self, controller: &mut VisibleShellController) {
        for _ in 0..64 {
            let s = controller.snapshot().expect("visible activity");
            if s.battle.is_some() {
                self.battle(controller);
            } else if s.phase == RuntimeShellPhase::Text
                || controller.has_pending_script_work()
                || controller.shell.field_notice.is_some()
                || controller.shell.visible_field_item_notice.is_some()
                || !controller.shell.battle_messages.is_empty()
            {
                controller.press(GameButton::A).expect("native text page");
                controller
                    .wait_frames(16)
                    .expect("native text presentation");
            } else {
                // Use the collected potion on an actually injured partner,
                // rather than assuming the lead took damage in a fixed fight.
                if self.medicine_uses == 0
                    && s.party
                        .slots
                        .iter()
                        .any(|p| p.pokemon.hp > 0 && p.pokemon.hp < p.pokemon.max_hp)
                    && s.bag
                        .items
                        .iter()
                        .any(|i| i.item_id == "SUPER_POTION" && i.quantity > 0)
                {
                    use_collected_super_potion(controller);
                    self.medicine_uses += 1;
                    continue;
                }
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
        let initial_moves: Vec<_> = initial
            .party
            .slots
            .iter()
            .flat_map(|p| {
                p.pokemon
                    .moves
                    .iter()
                    .map(move |m| (p.index, m.name.clone(), m.current_pp))
            })
            .collect();
        for _ in 0..768 {
            let s = controller.snapshot().expect("production battle");
            let Some(battle) = &s.battle else {
                break;
            };
            if !wild {
                self.trainer_opponents
                    .insert(battle.enemy_pokemon.species.id.clone());
            }
            if let Some(button) = super::rochester_earned::learning_button(controller, self.runtime)
            {
                controller
                    .press(button)
                    .expect("native move learning input");
                controller
                    .wait_frames(16)
                    .expect("native learning presentation");
                continue;
            }
            // Visible narration owns input even when a cursor exists behind it.
            if !controller.shell.battle_messages.is_empty() || s.ui.menu.is_none() {
                controller
                    .press(GameButton::A)
                    .expect("native battle narration");
                controller
                    .wait_frames(16)
                    .expect("native narration presentation");
                continue;
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
            let desired_species = match battle.enemy_pokemon.species.id.as_str() {
                "FLOSSAIL" => Some("NOTCHBILL"),
                "NOTCHBILL" | "TAPACITOR" => Some("ZAPSAP"),
                "AGATOT" | "KINDLIT" => Some("PORTABEAU"),
                _ => None,
            };
            let desired_slot = s.party.slots.iter().position(|p| {
                Some(p.pokemon.species.id.as_str()) == desired_species && p.pokemon.hp > 0
            });
            let switch_needed = desired_slot.is_some_and(|slot| {
                Some(slot) != battle.active_player_party_index
                    && battle.commands.switch_party_indices.contains(&slot)
            });
            let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
                // A replacement cursor may already exist behind the faint
                // message. Dialogue still owns input until its pages close.
                GameButton::A
            } else if options.iter().any(|o| o.contains("change <PKMN>")) {
                if controller
                    .shell
                    .battle_shift_prompt_cursor
                    .as_ref()
                    .expect("visible trainer Shift choice")
                    .option_index
                    == 0
                {
                    GameButton::Down
                } else {
                    GameButton::A
                }
            } else if options.iter().any(|o| o.contains("FIGHT")) {
                let target = if wild {
                    3
                } else if switch_needed {
                    1
                } else {
                    0
                };
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
                let target = desired_slot
                    .filter(|slot| battle.commands.switch_party_indices.contains(slot))
                    .or_else(|| {
                        s.party.slots.iter().position(|p| {
                            p.pokemon.hp > 0 && Some(p.index) != battle.active_player_party_index
                        })
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
                let target = super::rochester_earned::best_attack(self.runtime, battle, pokemon);
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
            if s.ui.menu.is_none() && s.ui.text.is_some() {
                controller
                    .wait_frames(30)
                    .expect("native battle presentation");
            }
            controller.press(button).expect("production battle input");
            controller
                .wait_frames(16)
                .expect("native battle input presentation");
        }
        let ended = controller.snapshot().expect("battle result");
        if std::env::var_os("CRYSTAL_ROUTE_TEST_TRACE").is_some() {
            eprintln!(
                "battle {:?}: party {:?}",
                initial.battle.as_ref().map(|b| &b.kind),
                ended
                    .party
                    .slots
                    .iter()
                    .map(|p| (
                        p.pokemon.species.id.as_str(),
                        p.pokemon.hp,
                        &p.pokemon.status
                    ))
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            ended.overworld.map_name,
            initial.overworld.map_name,
            "journey blacked out during {:?}",
            initial.battle.as_ref().map(|b| &b.kind)
        );
        assert!(
            ended.battle.is_none(),
            "battle did not end: turns={:?}, party_hp={:?}, ui={:?}",
            ended
                .battle
                .as_ref()
                .map(|b| (b.player_turns_taken, b.enemy_turns_taken)),
            ended
                .party
                .slots
                .iter()
                .map(|p| p.pokemon.hp)
                .collect::<Vec<_>>(),
            ended.ui
        );
        if !wild {
            assert!(
                ended.party.slots.iter().any(|p| p.pokemon.hp > 0),
                "local party lost against {:?}",
                initial.battle.as_ref().map(|b| &b.kind)
            );
            assert!(
                initial_moves.iter().any(|(slot, name, before)| {
                    ended.party.slots.iter().any(|p| {
                        p.index == *slot
                            && p.pokemon
                                .moves
                                .iter()
                                .any(|m| m.name == *name && m.current_pp < *before)
                    })
                }),
                "trainer battle never spent original move PP"
            );
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
                .is_some_and(|text| text.contains(expected))
                || controller
                    .shell
                    .visible_field_item_notice
                    .as_ref()
                    .is_some_and(|notice| notice.sound_trigger_text.contains(expected)),
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

    fn walk(&mut self, controller: &mut VisibleShellController, grid: &RouteGrid, to: Point) {
        let from = position(controller);
        for button in grid.path(from, to) {
            let before = position(controller);
            for _ in 0..8 {
                controller.press(button).expect("production route input");
                self.settle(controller);
                let snapshot = controller.snapshot().expect("settled step");
                if snapshot.overworld.map_name == "MnR05RockCut" {
                    assert!(
                        [(16, 77), (30, 27)].contains(&to),
                        "unexpected cave entrance"
                    );
                    return;
                }
                assert!(
                    snapshot.overworld.map_name.starts_with("MnR05C"),
                    "left route toward {to:?}: map={}, party={:?}",
                    snapshot.overworld.map_name,
                    snapshot
                        .party
                        .slots
                        .iter()
                        .map(|p| (p.pokemon.hp, &p.pokemon.status))
                        .collect::<Vec<_>>()
                );
                if position(controller) != before {
                    break;
                }
            }
            assert_ne!(
                position(controller),
                before,
                "blocked populated step {before:?} {button:?}"
            );
        }
        assert_eq!(position(controller), to);
    }

    fn talk(
        &mut self,
        controller: &mut VisibleShellController,
        grid: &RouteGrid,
        at: Point,
        expected: &str,
    ) {
        let (approach, button) = neighbors(at)
            .into_iter()
            .find(|(p, _)| {
                grid.floor.contains(p)
                    && !grid.blocked.contains(p)
                    && !grid.hazards.contains(p)
                    && p.1 / 48 == at.1 / 48
            })
            .expect("safe talk approach");
        self.walk(controller, grid, approach);
        self.read(controller, opposite(button), expected);
    }
}

fn use_collected_super_potion(controller: &mut VisibleShellController) {
    let before = controller.snapshot().expect("collected medicine");
    assert_eq!(
        before
            .bag
            .items
            .iter()
            .find(|i| i.item_id == "SUPER_POTION")
            .map_or(0, |i| i.quantity),
        1,
        "one actual route pickup"
    );
    super::authored_route_fixture::use_super_potion(controller);
    assert!(
        !controller
            .snapshot()
            .expect("consumed collected medicine")
            .bag
            .items
            .iter()
            .any(|i| i.item_id == "SUPER_POTION" && i.quantity > 0)
    );
}

fn rest_in_rock_cut(
    journey: &mut Journey,
    controller: &mut VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
) {
    indoor_walk(controller, runtime, (4, 13));
    let tired = controller.snapshot().expect("actual recovery need");
    assert!(
        tired.party.slots.iter().any(|p| {
            p.pokemon.hp < p.pokemon.max_hp
                || p.pokemon.status.is_some()
                || p.pokemon
                    .moves
                    .iter()
                    .any(|m| m.current_pp < runtime.data().moves[&m.name].pp)
        }),
        "native shelter must actually restore HP, status or spent PP"
    );
    journey.read(controller, GameButton::Right, "Rest a while");
    let rested = controller.snapshot().expect("rock-cut rest");
    for partner in &rested.party.slots {
        assert_eq!(
            partner.pokemon.hp, partner.pokemon.max_hp,
            "rest must restore HP"
        );
        assert!(partner.pokemon.status.is_none(), "rest must clear status");
        for learned in &partner.pokemon.moves {
            assert_eq!(
                learned.current_pp,
                runtime.data().moves[&learned.name].pp,
                "rest must restore PP"
            );
        }
    }
}

#[test]
#[ignore = "requires the external River Bluffs pack"]
fn river_bluffs_journey_wins_five_trainers_uses_rock_cut_and_medicine_and_continues() {
    let (root, runtime, mut controller) = fixture();
    // Explicit local setup: legal partners, four badges, position and replay
    // clock. This does not stand in for an earned regional Q03 journey.
    for (species, level) in [("PORTABEAU", 26), ("ZAPSAP", 25), ("NOTCHBILL", 25)] {
        let pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal partner");
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
    state.badges.johto = [true, true, true, true, false, false, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            std::iter::repeat_n(0, 16)
                .chain((0_u32..131072).map(|n| ((n * 73 + 19) % 256) as u8))
                .collect::<Vec<u8>>(),
        );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR05",
            columns: 1,
            rows: 3,
            tileset: "mn_headwaters",
            start: (22, 106),
            doors: BTreeMap::from([((16, 77), "MnR05RockCut"), ((30, 27), "MnR05RockCut")]),
        },
    );
    let mut journey = Journey {
        runtime: &runtime,
        wild_encounters: 0,
        trainer_opponents: BTreeSet::new(),
        attacks_spent_pp: 0,
        medicine_uses: 0,
    };
    journey.talk(&mut controller, &grid, (19, 103), "city gyms");
    journey.walk(&mut controller, &grid, (12, 89));
    assert!(
        journey.wild_encounters > 0,
        "shared entrance habitat never encountered wildlife"
    );
    journey.talk(&mut controller, &grid, (9, 89), "found");
    journey.talk(&mut controller, &grid, (11, 85), "warm");
    journey.walk(&mut controller, &grid, (55, 89));
    journey.talk(&mut controller, &grid, (57, 89), "river is fast");
    journey.talk(&mut controller, &grid, (46, 86), "RIVER PIER");
    journey.talk(&mut controller, &grid, (41, 81), "found");
    journey.talk(&mut controller, &grid, (31, 65), "wind");
    journey.walk(&mut controller, &grid, (37, 75));
    for _ in 0..8 {
        controller.press(GameButton::Down).expect("lower ledge");
        journey.settle(&mut controller);
        if position(&mut controller) == (37, 78) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (37, 78));
    controller.press(GameButton::Up).expect("reverse blocked");
    assert_eq!(position(&mut controller), (37, 78));
    for at in [(13, 78), (13, 75), (37, 75), (16, 77)] {
        journey.walk(&mut controller, &grid, at);
    }
    rest_in_rock_cut(&mut journey, &mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (5, 6));
    journey.read(&mut controller, GameButton::Up, "found");
    indoor_walk(&mut controller, &runtime, (11, 5));
    assert_eq!(position(&mut controller), (30, 28), "upper cave exit");
    journey.talk(&mut controller, &grid, (19, 33), "Grass");
    journey.talk(&mut controller, &grid, (33, 29), "head");
    // Return through ordinary input to restore actual spent PP/injuries.
    // Native teams need not reproduce the historical Johto party's faints.
    journey.walk(&mut controller, &grid, (30, 27));
    rest_in_rock_cut(&mut journey, &mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (11, 5));
    assert_eq!(position(&mut controller), (30, 28));
    journey.talk(&mut controller, &grid, (37, 19), "found");
    journey.walk(&mut controller, &grid, (51, 21));
    journey.talk(&mut controller, &grid, (53, 21), "old stone");
    journey.talk(&mut controller, &grid, (50, 18), "MISSISSIPPI");
    journey.talk(&mut controller, &grid, (29, 9), "warm");
    journey.walk(&mut controller, &grid, (24, 2));
    journey.talk(&mut controller, &grid, (26, 4), "ST CLOUD");
    journey.walk(&mut controller, &grid, (37, 39));
    for _ in 0..8 {
        controller.press(GameButton::Down).expect("upper ledge");
        journey.settle(&mut controller);
        if position(&mut controller) == (37, 42) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (37, 42));
    controller.press(GameButton::Up).expect("reverse blocked");
    assert_eq!(position(&mut controller), (37, 42));
    for at in [(15, 42), (15, 39), (37, 39), (30, 27)] {
        journey.walk(&mut controller, &grid, at);
    }
    indoor_walk(&mut controller, &runtime, (5, 17));
    assert_eq!(position(&mut controller), (16, 78), "lower cave exit");
    journey.walk(&mut controller, &grid, (22, 106));
    assert_eq!(journey.attacks_spent_pp, 5);
    assert_eq!(
        journey.medicine_uses, 1,
        "collected medicine must restore actual battle damage"
    );
    for species in [
        "AGATOT",
        "AUGRUB",
        "NOTCHBILL",
        "FLOSSAIL",
        "TAPACITOR",
        "TICKLET",
        "KINDLIT",
    ] {
        assert!(
            journey.trainer_opponents.contains(species),
            "missing {species}"
        );
    }
    let flags = (1..=5)
        .map(|n| format!("EVENT_MN_R05_TRAINER_{n:02}"))
        .chain((1..=4).map(|n| format!("EVENT_MN_R05_ITEM_{n:02}")))
        .collect::<Vec<_>>();
    for flag in &flags {
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
    let before = controller.snapshot().expect("completed local journey");
    assert!(before.party.slots.iter().any(|p| p.pokemon.hp > 0));
    let save = std::env::temp_dir().join(format!(
        "geothite-river-bluffs-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save journey");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), save.clone(), None)
        .expect("Continue");
    assert_eq!(position(&mut loaded), (22, 106));
    let after = loaded.snapshot().expect("continued journey");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    for flag in &flags {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("continued flag"),
            "{flag}"
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_RIVER_BLUFFS_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export journey save");
    }
    std::fs::remove_file(save).expect("remove temporary save");
    eprintln!(
        "River Bluffs: {} wild encounters, five trainer victories/seven opponents with spent PP, both cave directions, both native ledges and uphill returns, collected medicine used and party/bag/flags continued",
        journey.wild_encounters
    );
}
