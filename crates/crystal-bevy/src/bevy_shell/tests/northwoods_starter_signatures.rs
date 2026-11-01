// These regressions consume the published pack. No move probabilities, damage
// rules, learnsets or production animation programs are replaced by fixtures.

#[derive(Clone, Copy)]
struct StarterSignatureCase {
    actor: &'static str,
    middle: &'static str,
    move_id: &'static str,
    move_type: &'static str,
    power: u16,
    pp: u8,
    effect: &'static str,
    chance: u8,
    physical: bool,
}

const STARTER_SIGNATURE_CASES: [StarterSignatureCase; 3] = [
    StarterSignatureCase {
        actor: "CYPRIELLE",
        middle: "SLIPPERINA",
        move_id: "SLIPPER_WALTZ",
        move_type: "FAIRY",
        power: 75,
        pp: 15,
        effect: "CONFUSE_HIT",
        chance: 20,
        physical: false,
    },
    StarterSignatureCase {
        actor: "BOREALYNX",
        middle: "RIMEPELT",
        move_id: "RIMEBRAND",
        move_type: "FIRE",
        power: 85,
        pp: 10,
        effect: "BURN_HIT",
        chance: 20,
        physical: true,
    },
    StarterSignatureCase {
        actor: "VOYAGEAR",
        middle: "PORTABEAU",
        move_id: "PADDLE_SWEEP",
        move_type: "WATER",
        power: 80,
        pp: 15,
        effect: "ATTACK_DOWN_HIT",
        chance: 30,
        physical: true,
    },
];

fn starter_signature_pack_available() -> bool {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        assert!(
            std::env::var_os("NORTHWOODS_STARTER_REQUIRE_PACK").is_none(),
            "starter-signature verification requires CRYSTAL_RENDER_TEST_PACK"
        );
        return false;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    let installed = shell.runtime.data().moves.contains_key("RIMEBRAND")
        && shell.runtime.data().moves.contains_key("PADDLE_SWEEP");
    assert!(
        installed || std::env::var_os("NORTHWOODS_STARTER_REQUIRE_PACK").is_none(),
        "the explicitly required starter-signature overlay is absent"
    );
    installed
}

#[test]
fn northwoods_starter_signature_pack_preserves_roles_and_final_level36_learners() {
    use crate::core::battle::damage::is_physical_move;
    use crate::core::models::pokemon::Stat;

    if !starter_signature_pack_available() {
        return;
    }
    let shell = route36_overworld_shell_for_battle_render_regression();
    let data = shell.runtime.data();
    for case in STARTER_SIGNATURE_CASES {
        let mov = &data.moves[case.move_id];
        assert_eq!(
            (mov.move_type.as_str(), mov.power, mov.accuracy, mov.pp),
            (case.move_type, case.power, 100, case.pp),
            "{}",
            case.move_id
        );
        assert_eq!(
            (mov.effect.as_str(), mov.effect_chance),
            (case.effect, case.chance),
            "{}",
            case.move_id
        );
        assert_eq!(
            is_physical_move(&data.type_categories, mov).expect("installed category"),
            case.physical,
            "{} must use its authored damage stat",
            case.move_id
        );
        assert!(
            data.learnsets[case.actor]
                .iter()
                .any(|entry| entry.0 == 36 && entry.1 == case.move_id),
            "{} learns its signature at36",
            case.actor
        );
        assert!(
            !data.learnsets[case.middle]
                .iter()
                .any(|entry| entry.1 == case.move_id)
        );
        assert!(
            data.evolutions
                .entries_for(case.middle)
                .expect("middle evolution")
                .iter()
                .any(|entry| entry.species == case.actor
                    && entry.method == "LEVEL"
                    && entry.level == Some(36))
        );
        if case.move_id != "SLIPPER_WALTZ" {
            let learners = data
                .learnsets
                .iter()
                .flat_map(|(actor, learnset)| {
                    learnset
                        .iter()
                        .filter(move |entry| entry.1 == case.move_id)
                        .map(move |entry| (actor.as_str(), entry.0))
                })
                .collect::<Vec<_>>();
            assert_eq!(learners, [(case.actor, 36)], "exclusive final starter move");
        }
        if case.move_id == "PADDLE_SWEEP" {
            assert_eq!((mov.stat, mov.amount), (Some(Stat::Attack), Some(-1)));
        }
    }
}

fn starter_signature_owned_shell(actor: &str, level: u8) -> BevyRuntimeShell {
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    shell
        .shell
        .set_trainer_identity("CHRIS", 1)
        .expect("fixture owner");
    {
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon.fill(None);
        state.sync_party_from_storage();
    }
    shell
        .shell
        .add_party_pokemon(
            actor,
            level,
            None,
            None,
            "CHRIS",
            1,
            Dv::from_non_hp(15, 15, 15, 15),
        )
        .expect("healthy owned starter");
    shell
}

fn starter_signature_battle_controller(
    case: StarterSignatureCase,
    seed: u16,
) -> VisibleShellController {
    let mut shell = starter_signature_owned_shell(case.actor, 36);
    let data = shell.runtime.data();
    assert!(
        data.learnsets[case.actor]
            .iter()
            .any(|entry| entry.0 <= 36 && entry.1 == case.move_id)
    );
    let enemy_pp = data.moves["MIMIC"].pp;
    let pp = data.moves[case.move_id].pp;
    {
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("starter")
            .moves = vec![crate::core::models::LearnedMove {
            name: case.move_id.to_string(),
            current_pp: pp,
            pp_ups: 0,
        }];
        state.sync_party_from_storage();
    }
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .expect("authored encounter entry");
    let species = match &shell.shell.session().state().battle {
        crate::core::state::BattleMemory::StaticWild { species, .. } => species.clone(),
        _ => panic!("authored static encounter"),
    };
    // An explicit level100 opponent keeps secondary-effect observations alive
    // with genuine calculated stats; never inflate HP beyond the stat ceiling.
    let durable_foe = shell
        .runtime
        .data()
        .create_pokemon(&species, 100, Dv::default())
        .expect("legal durable battle fixture");
    {
        let state = shell.shell.session_mut().state_mut();
        let crate::core::state::BattleMemory::StaticWild {
            enemy_pokemon,
            enemy_party,
            level,
            ..
        } = &mut state.battle
        else {
            panic!("authored static encounter");
        };
        // Retain the authored origin/species and declare a legal high-level foe.
        *level = durable_foe.level;
        *enemy_pokemon = durable_foe;
        enemy_pokemon.moves = vec![crate::core::models::LearnedMove {
            name: "MIMIC".to_string(),
            current_pp: enemy_pp,
            pp_ups: 0,
        }];
        enemy_party[0] = enemy_pokemon.clone();
        state.script_runtime.active_battle_combat = None;
    }
    mark_runtime_snapshot_dirty(&mut shell);
    prepare_visible_battle_entry(&mut shell).expect("production encounter presentation");
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.visible_send_out_animation = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_player_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    sync_visible_battle_action_cursor(&mut shell);
    shell.shell.session_mut().state_mut().random_state =
        crate::core::random::CrystalRandomState::default();
    *shell.shell.session_mut().divider_mut_for_tests() =
        crate::core::random::RuntimeDividerSource::replay(deterministic_battle_divider_trace(
            0xabc1_u16.wrapping_add(seed),
        ));
    VisibleShellController { shell }
}

#[test]
fn northwoods_starter_signatures_execute_real_pp_damage_and_both_secondary_branches() {
    use crate::core::models::pokemon::Stat;

    if !starter_signature_pack_available() {
        return;
    }
    let mut evidence = Vec::new();
    for case in STARTER_SIGNATURE_CASES {
        let mut branches = [false; 2];
        for seed in 0..128_u16 {
            let mut controller = starter_signature_battle_controller(case, seed);
            let before = controller.snapshot().expect("production battle menu");
            let enemy_hp = before.battle.as_ref().expect("live foe").enemy_pokemon.hp;
            assert!(
                before
                    .ui
                    .menu
                    .as_ref()
                    .expect("command menu")
                    .layout
                    .vertical_menus[0]
                    .options
                    .iter()
                    .any(|option| option == ">FIGHT")
            );
            controller.press(GameButton::A).expect("FIGHT");
            let selected = controller.snapshot().expect("real move menu");
            assert_eq!(
                selected.battle.as_ref().expect("battle").player_moves[0].name,
                case.move_id
            );
            controller
                .press(GameButton::A)
                .expect("use selected signature");
            let after = controller.snapshot().expect("resolved production turn");
            let state = controller.shell.shell.session().state();
            let actor = state.storage.party.pokemon[0]
                .as_ref()
                .expect("owned starter");
            assert_eq!(
                actor.moves[0].current_pp,
                case.pp - 1,
                "{} real PP",
                case.move_id
            );
            let combat = state
                .script_runtime
                .active_battle_combat
                .as_ref()
                .expect("durable neutral foe keeps encounter live");
            let damage_event = controller.shell.last_audio_events.iter().any(|event| {
                event
                    .split_once(" events=")
                    .and_then(|(_, events)| events.split_whitespace().next())
                    .is_some_and(|events| events.split(',').any(|event| event == "player_damage"))
            });
            if !damage_event {
                continue;
            } // Preserve production accuracy, including Crystal's miss roll.
            assert!(
                combat.enemy.hp > 0 && combat.enemy.hp < enemy_hp,
                "{} must damage an actual surviving target",
                case.move_id
            );
            let applied = match case.move_id {
                "SLIPPER_WALTZ" => combat.enemy.confusion_turns > 0,
                "RIMEBRAND" => combat.enemy.status.as_deref() == Some("BURN"),
                "PADDLE_SWEEP" => combat.enemy.stat_boosts.get(&Stat::Attack) == Some(&-1),
                _ => unreachable!("fixed starter move cases"),
            };
            let label = if case.move_id == "SLIPPER_WALTZ" {
                crystal_assets::minnesota_signature_animations::animation_label(case.move_id)
            } else {
                crystal_assets::starter_signature_animations::animation_label(case.move_id)
            };
            assert!(
                controller
                    .shell
                    .visible_move_animations
                    .iter()
                    .any(|animation| animation.move_id == case.move_id
                        && animation.animation_label == label
                        && animation.total_frames > 1
                        && !animation.object_events.is_empty()),
                "{} must queue its installed native animation",
                case.move_id
            );
            let text = after
                .ui
                .text
                .as_ref()
                .expect("visible authoritative battle dialogue");
            assert_eq!(text.label, "visible-shell:battle-message");
            assert!(text.asm_text.as_ref().is_some_and(|text| !text.is_empty()));
            assert!(controller.shell.last_error.is_none());
            let branch = usize::from(applied);
            if !branches[branch] {
                evidence.push(serde_json::json!({"actor":case.actor,"level":36,
                    "move_id":case.move_id,"seed":seed,"pp_before":case.pp,
                    "pp_after":actor.moves[0].current_pp,"target_hp_before":enemy_hp,
                    "target_hp_after":combat.enemy.hp,"secondary_applied":applied,
                    "animation_label":label,"production_controller":true}));
            }
            branches[branch] = true;
            if branches == [true, true] {
                break;
            }
        }
        assert_eq!(
            branches,
            [true, true],
            "{} must preserve a real probabilistic secondary",
            case.move_id
        );
    }
    if let Some(path) = std::env::var_os("NORTHWOODS_STARTER_CONTROLLER_REPORT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&evidence).expect("serialize proof"),
        )
        .expect("retain production controller proof");
    }
}

#[test]
fn northwoods_starter_signatures_are_reachable_after_level36_evolution_and_forget_queue() {
    if !starter_signature_pack_available() {
        return;
    }
    for case in STARTER_SIGNATURE_CASES {
        for move_count in [1, 4] {
            let mut shell = starter_signature_owned_shell(case.middle, 35);
            {
                let state = shell.shell.session_mut().state_mut();
                let pokemon = state.storage.party.pokemon[0]
                    .as_mut()
                    .expect("middle starter");
                assert_eq!(pokemon.moves.len(), 4, "natural level35 moveset");
                drop(pokemon.moves.drain(..4 - move_count));
                state.sync_party_from_storage();
            }
            shell
                .shell
                .add_bag_item("RARE_CANDY", 1)
                .expect("native Rare Candy");
            let mut controller = VisibleShellController { shell };
            controller.press(GameButton::Start).expect("open Start");
            for _ in 0..10 {
                if visible_start_menu_entries(&controller.shell)
                    .expect("Start entries")
                    .iter()
                    .any(|entry| entry == ">PACK")
                {
                    break;
                }
                controller.press(GameButton::Down).expect("select PACK");
            }
            assert!(
                visible_start_menu_entries(&controller.shell)
                    .expect("PACK selected")
                    .iter()
                    .any(|entry| entry == ">PACK")
            );
            controller.press(GameButton::A).expect("open PACK");
            let index = controller
                .snapshot()
                .expect("bag")
                .bag
                .items
                .iter()
                .filter(|item| item.quantity > 0)
                .position(|item| item.item_id == "RARE_CANDY")
                .expect("carried Rare Candy");
            for _ in 0..index {
                controller.press(GameButton::Down).expect("select candy");
            }
            controller.press(GameButton::A).expect("item actions");
            controller.press(GameButton::A).expect("USE");
            assert_eq!(
                controller.shell.field_pack_target_mode,
                Some(FieldPackTargetMode::PartyPokemon)
            );
            controller
                .press(GameButton::A)
                .expect("grow actual middle starter to36");
            let mut signature_forget_prompt = false;
            let mut visible_evolution = false;
            let mut learn_trace = Vec::new();
            for _ in 0..192 {
                let snapshot = controller.snapshot().unwrap_or_else(|error| {
                    panic!("growth/evolution/learn state: {error:#}; actor={} initial_moves={} party_cursor={:?} forget={} decision={:?} trace={learn_trace:?}",
                        case.actor, move_count, controller.shell.party_move_cursor,
                        controller.shell.move_learn_forget_menu_open,
                        controller.shell.move_learn_decision)
                });
                if let Some(text) = snapshot
                    .ui
                    .text
                    .as_ref()
                    .and_then(|text| text.asm_text.as_ref())
                {
                    visible_evolution |= text.contains("evolved");
                }
                let state = controller.shell.shell.session().state();
                let trace = format!(
                    "pending={:?} queue={:?} notice={:?} notices={} forget={} decision={:?} boundary={:?}",
                    state
                        .pending_move_learn
                        .as_ref()
                        .map(|pending| &pending.learned_move.name),
                    state
                        .pending_move_learn_queue
                        .iter()
                        .map(|pending| &pending.learned_move.name)
                        .collect::<Vec<_>>(),
                    controller.shell.field_notice,
                    controller.shell.field_notice_queue.len(),
                    controller.shell.move_learn_forget_menu_open,
                    controller.shell.move_learn_decision,
                    controller
                        .shell
                        .special_boundary
                        .as_ref()
                        .map(|boundary| &boundary.label),
                );
                if learn_trace.last() != Some(&trace) {
                    learn_trace.push(trace);
                }
                signature_forget_prompt |= state
                    .pending_move_learn
                    .as_ref()
                    .is_some_and(|pending| pending.learned_move.name == case.move_id)
                    && controller.shell.move_learn_forget_menu_open;
                let done = state.pending_move_learn.is_none()
                    && state.pending_move_learn_queue.is_empty()
                    && controller.shell.field_notice.is_none()
                    && controller.shell.field_notice_queue.is_empty()
                    && controller.shell.field_evolution_cancellation.is_none();
                if done {
                    break;
                }
                // Keep naturally learned HMs. Read the actual menu selection
                // and move to a legal replacement using ordinary input.
                let protected_selection = controller.shell.move_learn_forget_menu_open
                    && controller.shell.special_boundary.is_none()
                    && controller.shell.battle_messages.is_empty()
                    && controller.shell.field_notice.is_none()
                    && state.pending_move_learn.as_ref().is_some_and(|pending| {
                        let selected = controller
                            .shell
                            .party_move_cursor
                            .as_ref()
                            .map(|cursor| cursor.option_index);
                        let learned = snapshot
                            .party
                            .slots
                            .iter()
                            .find(|slot| slot.index == pending.party_index)
                            .and_then(|slot| {
                                selected.and_then(|index| slot.pokemon.moves.get(index))
                            });
                        learned.is_some_and(|learned| {
                            snapshot.items.iter().any(|item| {
                                !item.consumable
                                    && item.tmhm_move.as_deref() == Some(learned.name.as_str())
                            })
                        })
                    });
                if protected_selection {
                    controller
                        .press(GameButton::Down)
                        .expect("keep HM and choose another slot");
                    continue;
                }
                let before_cursor = controller.shell.party_move_cursor.clone();
                controller.press(GameButton::A).unwrap_or_else(|error| {
                    panic!("accept actual evolution and move choices: {error:#}; actor={} initial_moves={} before_cursor={before_cursor:?} after_cursor={:?} forget={} decision={:?} trace={learn_trace:?} events={:?}",
                        case.actor, move_count, controller.shell.party_move_cursor,
                        controller.shell.move_learn_forget_menu_open, controller.shell.move_learn_decision,
                        controller.shell.last_audio_events.iter().rev().take(16).collect::<Vec<_>>())
                });
            }
            let after = controller.snapshot().expect("completed evolution");
            let pokemon = &after.party.slots[0].pokemon;
            assert_eq!(
                (pokemon.species.id.as_str(), pokemon.level),
                (case.actor, 36)
            );
            assert!(
                visible_evolution,
                "actual evolution dialogue must be presented"
            );
            assert!(
                pokemon
                    .moves
                    .iter()
                    .any(|mov| mov.name == case.move_id && mov.current_pp == case.pp),
                "{} must be learned after all earlier level36 choices; initial_moves={} actual_moves={:?} trace={:?}",
                case.move_id,
                move_count,
                pokemon.moves,
                learn_trace
            );
            if move_count == 4 {
                assert!(
                    signature_forget_prompt,
                    "{} actual four-move forget menu",
                    case.move_id
                );
            }
            let state = controller.shell.shell.session().state();
            assert!(
                state.pending_move_learn.is_none() && state.pending_move_learn_queue.is_empty()
            );
            assert_eq!(
                state
                    .bag
                    .quantity(&controller.shell.runtime.data().items["RARE_CANDY"]),
                0
            );
            assert!(controller.shell.last_error.is_none());
        }
    }
}

#[test]
fn northwoods_starter_forget_cursor_survives_battle_presentation_cleanup() {
    if !starter_signature_pack_available() {
        return;
    }
    let mut shell = starter_signature_owned_shell("RIMEPELT", 36);
    shell.shell.session_mut().state_mut().pending_move_learn =
        Some(crate::core::state::PendingMoveLearn {
            party_index: 0,
            species_id: "RIMEPELT".into(),
            level: 36,
            learned_move: crate::core::models::LearnedMove {
                name: "RIMEBRAND".into(),
                current_pp: 10,
                pp_ups: 0,
            },
            defer_level_evolution: false,
        });
    open_visible_pending_move_forget_menu(&mut shell).expect("open actual forget menu");
    let cursor = shell.party_move_cursor.clone();
    assert!(cursor.is_some());
    reset_visible_battle_exit_state(&mut shell);
    assert_eq!(
        shell.party_move_cursor, cursor,
        "pending move owns its cursor after battle cleanup"
    );
    assert!(shell.move_learn_forget_menu_open);
    let mut controller = VisibleShellController { shell };
    controller
        .snapshot()
        .expect("forget menu remains renderable");
}
