#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum CatchTutorialPhase {
    #[default]
    Narration,
    Fight,
    Pack,
    Items,
    Balls,
    Capture,
}

#[derive(Default)]
struct VisibleCatchTutorial {
    phase: CatchTutorialPhase,
    wait: u16,
}

fn tutorial_backpic_id(snapshot: &RuntimeShellSnapshot) -> &'static str {
    // GetPlayerOrMonPalettePointer still selects the player's gender palette
    // when GetTrainerBackpic substitutes DudeBackpic during the tutorial.
    if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
        "battle-player:dude_female"
    } else {
        "battle-player:dude"
    }
}

fn advance_visible_catch_tutorial(shell: &mut BevyRuntimeShell, ticks: u32) -> Result<()> {
    if shell.battle_action_cursor.is_none() {
        shell.battle_action_cursor = Some(MenuCursor {
            surface_id: "battle:actions".into(),
            option_index: 0,
        });
    }
    for _ in 0..ticks {
        let Some(tutorial) = shell.visible_catch_tutorial.as_mut() else {
            break;
        };
        if let Some(message) = shell.battle_messages.front() {
            if !visible_battle_message_is_complete(shell, message) {
                shell.visible_catch_tutorial.as_mut().unwrap().wait = 0;
                break;
            }
            let tutorial = shell.visible_catch_tutorial.as_mut().unwrap();
            tutorial.wait += 1;
            // DudeAutoInput_A: 80 no-input polls, then the automatic A press.
            if tutorial.wait > 80 {
                tutorial.wait = 0;
                press_visible_a_button(shell)?;
                if catch_tutorial_battle_active(shell) && shell.battle_action_cursor.is_none() {
                    shell.battle_action_cursor = Some(MenuCursor {
                        surface_id: "battle:actions".into(),
                        option_index: 0,
                    });
                }
                // Let the ordinary animation/text owners process the result.
                break;
            }
            continue;
        }
        tutorial.wait = tutorial.wait.saturating_add(1);
        let phase = tutorial.phase;
        let wait = tutorial.wait;
        let next = match phase {
            CatchTutorialPhase::Narration => Some(CatchTutorialPhase::Fight),
            // DownA's long $fe spans are busy-polled by Do2DMenuRTCJoypad,
            // not 254 VBlanks each. Retain a short visible menu pause for
            // each selection; never park the tutorial waiting for a human.
            CatchTutorialPhase::Fight if wait >= 16 => {
                let snapshot = shell.shell.snapshot()?;
                let battle = snapshot
                    .battle
                    .as_ref()
                    .context("tutorial battle disappeared")?;
                let option_index = visible_battle_action_ids(&snapshot, battle)
                    .iter()
                    .position(|action| *action == VisibleBattleAction::Pack)
                    .context("tutorial has no PACK command")?;
                shell.battle_action_cursor = Some(MenuCursor {
                    surface_id: "battle:actions".into(),
                    option_index,
                });
                Some(CatchTutorialPhase::Pack)
            }
            CatchTutorialPhase::Pack if wait >= 16 => {
                reset_visible_battle_item_cursors(shell);
                shell.bag_cursor = Some(MenuCursor {
                    surface_id: "battle:bag-items".into(),
                    option_index: 0,
                });
                Some(CatchTutorialPhase::Items)
            }
            CatchTutorialPhase::Items if wait >= 9 => {
                shell.bag_cursor = None;
                shell.ball_cursor = Some(MenuCursor {
                    surface_id: "bag:balls".into(),
                    option_index: 0,
                });
                Some(CatchTutorialPhase::Balls)
            }
            CatchTutorialPhase::Balls if wait >= 9 => {
                reset_visible_battle_item_cursors(shell);
                shell.visible_catch_tutorial.as_mut().unwrap().phase = CatchTutorialPhase::Capture;
                shell.visible_catch_tutorial.as_mut().unwrap().wait = 0;
                throw_visible_battle_ball_id(shell, 0, "POKE_BALL".into())?;
                break;
            }
            CatchTutorialPhase::Capture => {
                // Normal capture completion restores the map and resumes
                // the script after catchtutorial. Do not consume its dialogue.
                shell.visible_catch_tutorial = None;
                break;
            }
            _ => None,
        };
        if let Some(phase) = next {
            shell.visible_catch_tutorial = Some(VisibleCatchTutorial { phase, wait: 0 });
            mark_runtime_snapshot_dirty(shell);
            break;
        }
    }
    Ok(())
}

fn catch_tutorial_battle_active(shell: &BevyRuntimeShell) -> bool {
    matches!(&shell.shell.session().state().battle,
        crate::core::state::BattleMemory::Wild { battle_type, .. }
        | crate::core::state::BattleMemory::StaticWild { battle_type, .. }
        if battle_type == "BATTLETYPE_TUTORIAL")
}
