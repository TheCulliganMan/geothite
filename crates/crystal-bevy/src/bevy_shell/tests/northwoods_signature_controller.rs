// This exercises keyboard/controller ownership, not a direct effect call or a
// hand-queued animation. Fixture setup changes only the owned player's party.

#[test]
fn northwoods_signatures_execute_through_production_controller() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let probe = route36_overworld_shell_for_battle_render_regression();
    if !probe.runtime.data().moves.contains_key("MAGNETIC_RAIL") {
        return; // Ordinary core packs intentionally omit the regional add-on.
    }
    drop(probe);
    for (move_id, actor) in [
        ("MAGNETIC_RAIL", "TACONARCH"),
        ("BOG_SIPHON", "MOSQUITAN"),
        ("TIP_UP_STRIKE", "TIPULORD"),
        ("SLIPPER_WALTZ", "CYPRIELLE"),
        ("PIKE_DRILL", "MUSKETUNGE"),
    ] {
        assert_northwoods_controller_move_hits(
            move_id,
            actor,
            &crystal_assets::minnesota_signature_animations::animation_label(move_id),
            true,
        );
    }
}

fn assert_northwoods_controller_move_hits(
    move_id: &str,
    actor: &str,
    expected_animation_label: &str,
    require_object_events: bool,
) {
    assert_northwoods_controller_move_hits_at_level(
        move_id,
        actor,
        expected_animation_label,
        require_object_events,
        60,
    );
}

fn assert_northwoods_controller_move_hits_at_level(
    move_id: &str,
    actor: &str,
    expected_animation_label: &str,
    require_object_events: bool,
    level: u8,
) {
    let mut witnessed_hit = false;
    // Each attempt starts a fresh production encounter with a replayed
    // divider stream. This handles Magnetic Rail's real 95% accuracy
    // without changing the move or its production hit checks.
    for seed in 0..16_u16 {
        let mut shell = route36_overworld_shell_for_battle_render_regression();
        let pp = shell.runtime.data().moves[move_id].pp;
        assert!(
            shell.runtime.data().learnsets[actor]
                .iter()
                .any(|entry| entry.0 <= level && entry.1 == move_id),
            "{actor} must genuinely learn {move_id} in the supplied pack"
        );
        shell
            .shell
            .set_trainer_identity("CHRIS", 1)
            .expect("fixture trainer");
        {
            let state = shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0] = None;
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
            .expect("healthy, owned regional actor");
        {
            let state = shell.shell.session_mut().state_mut();
            let actor = state.storage.party.pokemon[0].as_mut().expect("actor");
            actor.moves = vec![crate::core::models::LearnedMove {
                name: move_id.to_string(),
                current_pp: pp,
                pp_ups: 0,
            }];
            state.sync_party_from_storage();
        }
        shell
            .shell
            .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
            .expect("authored Sudowoodo encounter");
        prepare_visible_battle_entry(&mut shell).expect("production battle presentation");
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
        let mut controller = VisibleShellController { shell };
        let before = controller.snapshot().expect("production command menu");
        let owned = &before.party.slots[0].pokemon;
        assert_eq!(owned.species.id, actor);
        assert_eq!(owned.original_trainer_name, before.trainer.player_name);
        assert_eq!(owned.original_trainer_id, before.trainer.player_id);
        assert_eq!(owned.hp, owned.max_hp);
        let enemy_hp = before.battle.as_ref().expect("battle").enemy_pokemon.hp;
        let command_options = &before
            .ui
            .menu
            .as_ref()
            .expect("command menu")
            .layout
            .vertical_menus[0]
            .options;
        assert!(command_options.iter().any(|option| option == ">FIGHT"));
        controller.press(GameButton::A).expect("press A on FIGHT");
        let selected = controller.snapshot().expect("move menu");
        assert!(selected.ui.menu.is_some());
        assert_eq!(
            selected.battle.as_ref().expect("battle").player_moves[0].name,
            move_id
        );
        controller
            .press(GameButton::A)
            .expect("press A on the authored move");
        let after = controller.snapshot().expect("resolved controller turn");
        let state = controller.shell.shell.session().state();
        let owned = state.storage.party.pokemon[0]
            .as_ref()
            .expect("authoritative actor");
        assert_eq!(
            owned.moves[0].current_pp,
            pp - 1,
            "{move_id} must spend real PP"
        );
        let final_hp_reduced = match &state.battle {
            crate::core::state::BattleMemory::StaticWild { enemy_pokemon, .. } => {
                enemy_pokemon.hp < enemy_hp
            }
            crate::core::state::BattleMemory::Inactive => {
                state
                    .pending_static_wild_terminal
                    .as_ref()
                    .is_some_and(|terminal| terminal.battle_result & 0x3f == 0)
                    || (owned.hp > 0
                        && controller
                            .shell
                            .battle_messages
                            .iter()
                            .any(|message| message.contains("fainted!")))
            }
            other => panic!("authored move changed the encounter kind: {other:?}"),
        };
        // The actual scaled encounter can answer Bog Siphon with Rest.
        // Its final HP then correctly returns to full. Require the real
        // Damage event and its generated intermediate HP scene, rather
        // than mistaking a successfully healed hit for an accuracy miss.
        let queued_hp_reduced = controller
            .shell
            .battle_message_scenes
            .iter()
            .chain(
                controller
                    .shell
                    .pending_battle_scenes_after_message
                    .iter()
                    .map(|(_, scene)| scene),
            )
            .filter_map(|scene| scene.battle.as_ref())
            .any(|battle| battle.enemy_pokemon.hp < enemy_hp);
        let damage_event_recorded = controller.shell.last_audio_events.iter().any(|event| {
            event
                .split_once(" events=")
                .and_then(|(_, events)| events.split_whitespace().next())
                .is_some_and(|events| events.split(',').any(|event| event == "player_damage"))
        });
        let damaged = damage_event_recorded && (final_hp_reduced || queued_hp_reduced);
        if !damaged {
            if std::env::var_os("NORTHWOODS_CONTROLLER_TRACE").is_some() {
                eprintln!(
                    "{move_id} seed={seed} initial_enemy_hp={enemy_hp} player_hp={}/{} battle={:?} combat={:?} messages={:?} events={:?}",
                    owned.hp,
                    owned.max_hp,
                    state.battle,
                    state
                        .script_runtime
                        .active_battle_combat
                        .as_ref()
                        .map(|combat| (
                            combat.player.hp,
                            combat.enemy.hp,
                            combat.enemy.max_hp,
                            &combat.enemy.moves,
                            combat.player_last_move.as_ref(),
                        )),
                    controller.shell.battle_messages,
                    controller.shell.last_audio_events,
                );
            }
            continue;
        }
        assert!(
            controller
                .shell
                .visible_move_animations
                .iter()
                .any(|animation| {
                    animation.move_id == move_id
                        && animation.animation_label == expected_animation_label
                        && animation.total_frames > 1
                        && (!animation.object_events.is_empty()
                            || (!require_object_events && !animation.bg_events.is_empty()))
                }),
            "{move_id} expected {expected_animation_label} but queued {:?}",
            controller
                .shell
                .visible_move_animations
                .iter()
                .map(|animation| (
                    &animation.move_id,
                    &animation.animation_label,
                    animation.total_frames,
                    animation.object_events.len(),
                    animation.bg_events.len()
                ))
                .collect::<Vec<_>>()
        );
        let text = after.ui.text.as_ref().expect("visible battle dialogue");
        assert_eq!(text.label, "visible-shell:battle-message");
        assert!(text.asm_text.as_ref().is_some_and(|text| !text.is_empty()));
        assert!(controller.shell.last_error.is_none());
        witnessed_hit = true;
        break;
    }
    assert!(
        witnessed_hit,
        "no real controller hit observed for {move_id}"
    );
}
