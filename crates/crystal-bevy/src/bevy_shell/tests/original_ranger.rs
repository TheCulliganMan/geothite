//! Placement and party are fixtures; talk, battle input and save use production paths.
use bevy::prelude::{Assets, Image};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, PokemonSpecies};
use crystal_core::state::{BattleMemory, OverworldMemory};
use crystal_core::world::map::Direction;
use serde_json::Value;
use std::cmp::Reverse;

use super::super::{BevyRuntimeShell, VisibleShellController};
use super::super::{
    load_gbcpal_palette, load_oak_intro_frame, load_sprite_art, mark_runtime_snapshot_dirty,
    visible_field_dialog_text,
};
use super::interior_life::{fixture, load};
use super::{menu_render_test_app, save_live_battle_canvas_for_test};

fn capture_battle(controller: VisibleShellController, name: &str) -> VisibleShellController {
    if std::env::var_os("POKEGEAR_PC_RENDER_DIR").is_none() {
        return controller;
    }
    let mut app = menu_render_test_app(controller.shell);
    app.update();
    save_live_battle_canvas_for_test(app.world_mut(), name);
    VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("retain production controller after capture"),
    }
}

fn capture_overworld(controller: VisibleShellController) -> VisibleShellController {
    let Some(directory) = std::env::var_os("POKEGEAR_PC_RENDER_DIR") else {
        return controller;
    };
    let mut app = menu_render_test_app(controller.shell);
    app.update();
    let world = app.world_mut();
    let images = world
        .remove_resource::<Assets<Image>>()
        .expect("owned CPU art");
    let canvas = super::pokemon_overworld_context_canvas(world, &images);
    world.insert_resource(images);
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).unwrap();
    canvas
        .save(directory.join("original-npc-overworld-state.png"))
        .unwrap();
    VisibleShellController {
        shell: world
            .remove_resource::<BevyRuntimeShell>()
            .expect("retain controller after map capture"),
    }
}

// Explicit prerequisite setup is separate from the observed victory state.
fn character_victory_recorded(c: &VisibleShellController, row: &Value) -> bool {
    let state = c.shell.shell.session().state();
    if let Some(flag) = row["defeated_flag"].as_str() {
        return state
            .flags
            .is_event_flag_set(flag)
            .expect("registered defeat flag");
    }
    let expected = row["completion_memory"]
        .as_object()
        .expect("explicit completion contract");
    assert!(!expected.is_empty());
    expected.iter().all(|(key, value)| {
        state.script_runtime.memory.get(key).map(String::as_str) == value.as_str()
    })
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published original ranger pack"]
fn original_npc_controller_talk_battle_turn_and_save() {
    let (_, runtime, _) = load();
    let root = runtime
        .materialize_runtime_files()
        .expect("owned mount of the actual published pack art");
    let source_path = std::env::var_os("CRYSTAL_ORIGINAL_NPC_SOURCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/npcs/sprites/01-northwoods-ranger/placement-v1.json",
            )
        });
    let source: Value = serde_json::from_slice(&std::fs::read(source_path).unwrap()).unwrap();
    assert_eq!(
        source["trainers"].as_array().unwrap().len(),
        1,
        "verify one NPC at a time"
    );
    let row = &source["trainers"][0];
    let id = row["id"].as_str().unwrap();
    let class = row["trainer_class"].as_str().unwrap();
    let sprite = row["sprite"].as_str().unwrap();
    let actor_script = row["actor_script"].as_str().unwrap_or(id);
    let stem = class.to_ascii_lowercase();
    let palette = load_gbcpal_palette(
        &root
            .runtime_assets()
            .join(format!("gfx/sprites/{stem}.gbcpal")),
    )
    .unwrap();
    let visible_light = [palette[1][0], palette[1][1], palette[1][2], 255];
    let mut images = Assets::<Image>::default();
    let overworld_art = load_sprite_art(&root, &stem, 0, "day", &mut images)
        .expect("original NPC six-pose art and own palette load through production renderer");
    for direction in [
        &overworld_art.down,
        &overworld_art.up,
        &overworld_art.left,
        &overworld_art.right,
    ] {
        assert!(direction.walking.is_some());
        let image = images
            .get(&direction.standing.handle)
            .expect("standing frame");
        let pixels = image.data.as_ref().expect("CPU review pixels");
        assert!(
            pixels.as_chunks::<4>().0.contains(&visible_light),
            "authored light palette survives map rendering"
        );
    }
    let battle_art = load_oak_intro_frame(&root, &format!("battle-trainer:{stem}"), &mut images)
        .expect("original trainer battle art loads through production renderer");
    let battle_pixels = images
        .get(&battle_art.handle)
        .expect("battle image")
        .data
        .as_ref()
        .expect("CPU battle pixels");
    assert!(
        battle_pixels.as_chunks::<4>().0.contains(&visible_light),
        "visible skin survives battle transparency"
    );
    {
        let map = row["map"].as_str().unwrap();
        let approach = (
            row["approach"][0].as_i64().unwrap() as i16,
            row["approach"][1].as_i64().unwrap() as i16,
        );
        let mut c = fixture(&root, &runtime, map, approach);
        let facing = match row["facing"].as_str().unwrap() {
            "west" => Direction::Right,
            "east" => Direction::Left,
            "north" => Direction::Down,
            "south" => Direction::Up,
            _ => unreachable!(),
        };
        {
            let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
            for (index, species) in ["BOREALYNX", "CYPRIELLE", "VOYAGEAR"]
                .into_iter()
                .enumerate()
            {
                state.storage.party.pokemon[index] = Some(
                    runtime
                        .data()
                        .create_pokemon(species, 60, Dv::from_non_hp(15, 15, 15, 15))
                        .unwrap(),
                );
            }
            state.sync_party_from_storage();
            if let Some(count) = row["fixture_badges"].as_u64() {
                assert!(count <= 8);
                state.badges.johto = std::array::from_fn(|index| (index as u64) < count);
                crystal_core::systems::script_flags::reconcile_saved_badge_flags(state);
            }
            if let Some(reward) = row["reward_item"].as_str() {
                assert_eq!(
                    state.bag.quantity(&runtime.data().items[reward]),
                    0,
                    "fixture must not award the new TM"
                );
            }
            if let Some(events) = row["fixture_events"].as_array() {
                for event in events {
                    state
                        .flags
                        .set_event_flag(event.as_str().expect("fixture flag"), true)
                        .expect("registered prerequisite flag");
                }
            }
            if let Some(memory) = row["fixture_memory"].as_object() {
                for (key, value) in memory {
                    state
                        .script_runtime
                        .memory
                        .insert(key.clone(), value.as_str().expect("fixture memory").into());
                }
            }
            overworld.set_player_facing(facing);
            state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
        }
        assert!(
            !character_victory_recorded(&c, row),
            "fixture must not award this victory"
        );
        mark_runtime_snapshot_dirty(&mut c.shell);
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap();
        assert!(
            visible_field_dialog_text(&before, &c.shell).is_some(),
            "trainer must introduce the battle: {id}"
        );
        c.press(GameButton::Down).unwrap();
        assert_eq!(
            c.snapshot().unwrap().overworld.tile,
            before.overworld.tile,
            "dialogue owns movement"
        );
        for _ in 0..200 {
            if c.snapshot().unwrap().battle.is_some() {
                break;
            }
            c.press(GameButton::A).unwrap();
        }
        let state = c.shell.shell.session().state();
        let trainer = runtime.data().trainers.get(id).unwrap();
        assert_eq!(trainer.trainer_class, class);
        assert!(
            runtime.data().maps[map]
                .objects
                .iter()
                .any(|object| object.sprite == sprite && object.script == actor_script)
        );
        let BattleMemory::Trainer {
            enemy_party: enemy, ..
        } = &state.battle
        else {
            panic!("production trainer battle must start: {id}");
        };
        let expected_level = if runtime.data().minnesota_trainer_scaling {
            60
        } else {
            trainer
                .party
                .iter()
                .map(|member| member.level)
                .max()
                .unwrap()
        };
        assert_eq!(
            enemy.iter().map(|mon| mon.level).max(),
            Some(expected_level)
        );
        c.wait_frames(60).unwrap();
        c = capture_battle(c, "original-npc-trainer-intro.png");
        let path = std::env::temp_dir().join(format!(
            "original-npc-{}-{id}.crystalsave",
            std::process::id()
        ));
        c.save(&path).unwrap();
        let saved = runtime.load_save(&path).unwrap();
        assert_eq!(
            saved.battle,
            c.shell.shell.session().state().battle,
            "evolved team survives save/load"
        );
        for _ in 0..200 {
            let s = c.snapshot().unwrap();
            if s.ui.menu.as_ref().is_some_and(|menu| {
                menu.layout
                    .vertical_menus
                    .iter()
                    .any(|menu| menu.options.iter().any(|entry| entry == ">FIGHT"))
            }) {
                break;
            }
            c.press(GameButton::A).unwrap();
        }
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap().party.slots[0].pokemon.moves.clone();
        c.press(GameButton::A).unwrap();
        let after = c.snapshot().unwrap();
        assert!(
            after.party.slots[0]
                .pokemon
                .moves
                .iter()
                .zip(before)
                .any(|(new, old)| new.current_pp < old.current_pp),
            "production battle executes the selected move: {id}"
        );
        assert!(!c.shell.battle_messages.is_empty());
        c = capture_battle(c, "original-npc-first-turn.png");

        let mut trace_budget = if std::env::var_os("CRYSTAL_NPC_BATTLE_TRACE").is_some() {
            80
        } else {
            0
        };
        let mut previous_trace = String::new();
        for _ in 0..4096 {
            let snapshot = c.snapshot().unwrap();
            let Some(battle) = snapshot.battle.as_ref() else {
                break;
            };
            if trace_budget > 0 {
                let trace = format!(
                    "active={:?} enemy={} hp={} last={:?} party={:?}",
                    battle.active_player_party_index,
                    battle.enemy_pokemon.species.id,
                    battle.enemy_pokemon.hp,
                    battle.enemy_last_move,
                    snapshot
                        .party
                        .slots
                        .iter()
                        .map(|slot| (slot.index, slot.pokemon.hp, slot.pokemon.perish_song_turns))
                        .collect::<Vec<_>>()
                );
                if trace != previous_trace {
                    eprintln!("NPC battle: {trace}");
                    previous_trace = trace;
                    trace_budget -= 1;
                }
            }
            let perish_escape = snapshot.party.slots.iter().any(|slot| {
                Some(slot.index) == battle.active_player_party_index
                    && slot.pokemon.hp > 0
                    && matches!(slot.pokemon.perish_song_turns, 1 | 2)
            }) && snapshot.party.slots.iter().any(|slot| {
                Some(slot.index) != battle.active_player_party_index && slot.pokemon.hp > 0
            });
            // Switch through the real PKMN menu when the lead is weak to the
            // opponent's STAB and a living partner resists it. The fixture party
            // and opponent remain unchanged; only legal controller inputs act.
            let vulnerability = |species: &PokemonSpecies| {
                let mut types = vec![species.type1.clone()];
                if species.type2 != species.type1
                    && !species.type2.is_empty()
                    && species.type2 != "NONE"
                {
                    types.push(species.type2.clone());
                }
                [
                    &battle.enemy_pokemon.species.type1,
                    &battle.enemy_pokemon.species.type2,
                ]
                .into_iter()
                .filter(|kind| !kind.is_empty() && kind.as_str() != "NONE")
                .map(|kind| {
                    let multiplier =
                        crystal_core::battle::damage::calculate_type_effectiveness_multiplier(
                            &runtime.data().type_effectiveness,
                            kind,
                            &types,
                        )
                        .expect("native defensive type matchups");
                    100_u64 * u64::from(multiplier.numerator) / u64::from(multiplier.denominator)
                })
                .max()
                .expect("opponent has a native type")
            };
            let mut defender_types = vec![battle.enemy_pokemon.species.type1.clone()];
            let secondary = &battle.enemy_pokemon.species.type2;
            if secondary != &defender_types[0] && !secondary.is_empty() && secondary != "NONE" {
                defender_types.push(secondary.clone());
            }
            let attack_score = |attacker: &PokemonSpecies, movement_name: &str| {
                let Some(data) = runtime.data().moves.get(movement_name) else {
                    return 0;
                };
                if data.power == 0 {
                    return 0;
                }
                let effectiveness =
                    crystal_core::battle::damage::calculate_type_effectiveness_multiplier(
                        &runtime.data().type_effectiveness,
                        &data.move_type,
                        &defender_types,
                    )
                    .expect("native offensive type matchups");
                let stab = if data.move_type == attacker.type1 || data.move_type == attacker.type2 {
                    150_u64
                } else {
                    100
                };
                let accuracy = if data.accuracy == 0 {
                    100
                } else {
                    data.accuracy
                };
                let mut score = u64::from(data.power)
                    * u64::from(accuracy)
                    * stab
                    * u64::from(effectiveness.numerator)
                    / u64::from(effectiveness.denominator);
                // Charging spends a turn without damage and exposes the user to
                // another enemy attack. Prefer a reliable immediate hit here.
                if data.effect == "SOLARBEAM" {
                    score /= 4;
                }
                score
            };
            let safest_partner = snapshot
                .party
                .slots
                .iter()
                .filter(|slot| {
                    slot.pokemon.hp > 0
                        && (!perish_escape || Some(slot.index) != battle.active_player_party_index)
                })
                .map(|slot| {
                    let offense = slot
                        .pokemon
                        .moves
                        .iter()
                        .filter(|movement| movement.current_pp > 0)
                        .map(|movement| attack_score(&slot.pokemon.species, &movement.name))
                        .max()
                        .unwrap_or(0);
                    (
                        slot.index,
                        vulnerability(&slot.pokemon.species),
                        offense,
                        slot.pokemon.hp,
                    )
                })
                .min_by_key(|(index, risk, offense, hp)| {
                    (*risk, Reverse(*offense), Reverse(*hp), *index)
                })
                .expect("living battle partner");
            let should_switch = perish_escape
                || snapshot
                    .party
                    .slots
                    .iter()
                    .find(|slot| Some(slot.index) == battle.active_player_party_index)
                    .is_some_and(|slot| {
                        let risk = vulnerability(&slot.pokemon.species);
                        risk >= 200 && safest_partner.1 < risk
                    });
            let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
                c.wait_frames(30).unwrap();
                GameButton::A
            } else if snapshot.ui.menu.as_ref().is_some_and(|menu| {
                menu.layout
                    .vertical_menus
                    .iter()
                    .flat_map(|m| &m.options)
                    .any(|entry| entry.contains("change <PKMN>"))
            }) {
                GameButton::B
            } else if let Some(cursor) = &c.shell.battle_faint_prompt_cursor {
                if cursor.option_index > 0 {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &c.shell.battle_switch_cursor {
                let target = safest_partner.0;
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &c.shell.battle_party_action_cursor {
                if cursor.option_index > 0 {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &c.shell.battle_move_cursor {
                let attacker = &snapshot
                    .party
                    .slots
                    .iter()
                    .find(|slot| Some(slot.index) == battle.active_player_party_index)
                    .expect("active battle partner")
                    .pokemon
                    .species;
                let safe_stall =
                    if battle.enemy_last_move.as_deref() == Some("DESTINY_BOND") {
                        battle
                            .player_moves
                            .iter()
                            .enumerate()
                            .find(|(_, movement)| {
                                movement.current_pp > 0
                                    && runtime.data().moves.get(&movement.name).is_some_and(
                                        |data| {
                                            data.power == 0
                                                || attack_score(attacker, &movement.name) == 0
                                                || (data.effect == "SUPER_FANG"
                                                    && battle.enemy_pokemon.hp > 1)
                                        },
                                    )
                            })
                            .map(|(index, _)| index)
                    } else {
                        None
                    };
                let target = safe_stall
                    .or_else(|| {
                        battle
                            .player_moves
                            .iter()
                            .enumerate()
                            .filter(|(_, movement)| movement.current_pp > 0)
                            .map(|(index, movement)| {
                                (index, attack_score(attacker, &movement.name))
                            })
                            .filter(|(_, score)| *score > 0)
                            .max_by_key(|(_, power)| *power)
                            .map(|(index, _)| index)
                    })
                    .expect("legal move with PP");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &c.shell.battle_action_cursor {
                match (should_switch, cursor.option_index) {
                    (true, 0) => GameButton::Right,
                    (true, 1) | (false, 0) => GameButton::A,
                    (false, 1) => GameButton::Left,
                    (_, 2 | 3) => GameButton::Up,
                    _ => panic!("invalid battle action"),
                }
            } else {
                GameButton::A
            };
            c.press(button).unwrap();
            c.wait_frames(16).unwrap();
        }
        assert!(
            c.snapshot().unwrap().battle.is_none(),
            "ranger battle must finish"
        );
        for _ in 0..160 {
            if visible_field_dialog_text(&c.snapshot().unwrap(), &c.shell).is_none()
                && character_victory_recorded(&c, row)
            {
                break;
            }
            c.press(GameButton::A).unwrap();
            c.wait_frames(16).unwrap();
        }
        let recorded_win_before_capture = character_victory_recorded(&c, row);
        c = capture_overworld(c);
        let terminal = c.snapshot().unwrap();
        assert!(
            recorded_win_before_capture,
            "winning records the ranger's defeated flag; map={} party={:?} messages={:?}",
            terminal.overworld.map_name,
            terminal
                .party
                .slots
                .iter()
                .map(|slot| (&slot.pokemon.species.id, slot.pokemon.hp))
                .collect::<Vec<_>>(),
            c.shell.battle_messages,
        );
        if let Some(slot) = row["badge_slot"].as_u64() {
            assert!(
                c.shell.shell.session().state().badges.johto[slot as usize - 1],
                "actual boss victory awards its native badge"
            );
        }
        if let Some(reward) = row["reward_item"].as_str() {
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .bag
                    .quantity(&runtime.data().items[reward]),
                1,
                "actual boss victory awards exactly one TM"
            );
        }
        c.press(GameButton::A).unwrap();
        assert!(
            visible_field_dialog_text(&c.snapshot().unwrap(), &c.shell).is_some(),
            "ranger remains talkable after the win"
        );
        assert!(
            c.snapshot().unwrap().battle.is_none(),
            "post-battle talk must not restart the battle"
        );
        if let Some(reward) = row["reward_item"].as_str() {
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .bag
                    .quantity(&runtime.data().items[reward]),
                1,
                "repeat talk must not duplicate the TM"
            );
        }
        c.save(&path).unwrap();
        let saved = runtime.load_save(&path).unwrap();
        assert_eq!(
            saved.flags,
            c.shell.shell.session().state().flags,
            "victory flags survive Continue"
        );
        assert_eq!(
            saved.badges,
            c.shell.shell.session().state().badges,
            "badges survive Continue"
        );
        assert_eq!(
            saved.script_runtime.memory,
            c.shell.shell.session().state().script_runtime.memory,
            "League progress survives Continue"
        );
        let _controller = capture_overworld(c);

        std::fs::remove_file(&path).unwrap();
        let _ = std::fs::remove_file(crystal_core::save::save_backup_path(&path));
    }
}
