// Human participants follow the same retained snapshot and entry/result clocks
// as classic portraits. They never advance narration or a send-out timer.
fn immersive_battle_trainers(
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    shell: &BevyRuntimeShell,
) -> [Option<crystal_render_api::VisualBattleTrainer>; 2] {
    let player_source = if battle.battle_type == "BATTLETYPE_TUTORIAL" {
        "outdoorsman"
    } else if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
        "kris"
    } else {
        "chris"
    };
    let player = crystal_render_api::VisualBattleTrainer {
        source_id: Arc::from(player_source),
        visible: shell.visible_battle_sliding_intro.is_some()
            || shell.battle_entry_messages_remaining > 0
            || battle.battle_type == "BATTLETYPE_TUTORIAL"
            || shell
                .visible_trainer_exit_animation
                .as_ref()
                .is_some_and(|exit| exit.side == crate::core::battle::turn::BattleSide::Player),
    };
    let enemy = match &battle.kind {
        RuntimeBattleKind::Trainer { trainer_class, .. } => {
            Some(crystal_render_api::VisualBattleTrainer {
                source_id: Arc::from(normalize_battle_trainer_sprite_id(trainer_class)),
                visible: shell.visible_battle_sliding_intro.is_some()
                    || shell.battle_entry_messages_remaining >= 3
                    || visible_trainer_result_frame(shell).is_some()
                    || shell
                        .visible_trainer_exit_animation
                        .as_ref()
                        .is_some_and(|exit| {
                            exit.side == crate::core::battle::turn::BattleSide::Enemy
                        }),
            })
        }
        _ => None,
    };
    [Some(player), enemy]
}
