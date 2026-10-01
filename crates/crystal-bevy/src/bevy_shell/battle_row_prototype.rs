// Experimental until native actor silhouettes, alpha edges and source OAM
// allocation/budget are independently verified. Production fallback stays on.
fn immersive_row_prototype_enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("CRYSTAL_BATTLE_ROW_PROTOTYPE").as_deref() == Ok("1"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

fn immersive_row_prototype_supported(
    animation: &VisibleMoveAnimation,
    battlers: &[Option<VisualBattleBattler>; 2],
) -> bool {
    let allowed: &[&str] = match (
        animation.move_id.as_str(),
        animation.animation_label.as_str(),
    ) {
        ("TACKLE", "BattleAnim_Tackle") => &[
            "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW",
            "BATTLE_BG_EFFECT_TACKLE",
            "BATTLE_BG_EFFECT_SHOW_MON",
        ],
        ("WATER_GUN", "BattleAnim_WaterGun") => &[
            "BATTLE_BG_EFFECT_START_WATER",
            "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW",
            "BATTLE_BG_EFFECT_WATER",
            "BATTLE_BG_EFFECT_SHOW_MON",
            "BATTLE_BG_EFFECT_END_WATER",
        ],
        _ => return false,
    };
    let expected_spawns = if animation.move_id == "TACKLE" { 1 } else { 3 };
    let row_effect = animation
        .bg_events
        .iter()
        .find(|effect| effect.effect_id == "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW");
    animation.started
        && row_effect.is_some_and(|effect| {
            effect.target
                == if animation.move_id == "TACKLE" {
                    "BG_EFFECT_TARGET"
                } else {
                    "BG_EFFECT_USER"
                }
        })
        && animation
            .object_events
            .iter()
            .filter(|event| matches!(event.command, VisibleMoveObjectCommand::Spawn { .. }))
            .count()
            == expected_spawns
        && animation
            .object_events
            .iter()
            .filter(|event| {
                matches!(
                    event.command,
                    VisibleMoveObjectCommand::Increment { index: 1 }
                )
            })
            .count()
            == 1
        && animation
            .object_events
            .iter()
            .all(|event| row_effect.is_some_and(|effect| event.frame > effect.frame))
        && battlers.iter().all(|battler| {
            battler
                .as_ref()
                .is_some_and(|b| b.visible && b.allow_species_model && !b.shiny)
        })
        && animation
            .bg_events
            .iter()
            .all(|effect| !effect.incremented && allowed.contains(&effect.effect_id.as_str()))
        && animation
            .bg_events
            .iter()
            .filter(|effect| effect.effect_id == "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW")
            .count()
            == 1
        && animation.object_events.iter().all(|event| {
            matches!(
                event.command,
                VisibleMoveObjectCommand::Spawn { .. }
                    | VisibleMoveObjectCommand::Increment { index: 1 }
            )
        })
}

/// Rendering a whole horizontal band is valid only when the current source
/// OAM contains the complete expected rectangle. Pressure/partial frames retain
/// the classic per-piece renderer instead of silently dropping coverage.
fn immersive_row_prototype_oam_supported(
    live: &[Option<VisibleBattleBattlerRowFrame>; 10],
) -> bool {
    live.iter().flatten().count() <= 1
        && live.iter().flatten().all(|row| {
            let columns = if row.player_side { 6 } else { 7 };
            row.row_count == 2
                && row.oam.entries.len() == columns * 2
                && row.oam.rows.len() == row.oam.entries.len()
                && row.oam.rows.iter().flatten().all(|visible| *visible)
        })
}

/// Consume live source OAM, including the final OAM emitted in the NULL
/// callback's deinitialization tick. Source callback state is the lifetime
/// authority; the renderer does not independently infer retirement.
fn immersive_row_prototype_rows(
    animation: &VisibleMoveAnimation,
    live: &[Option<VisibleBattleBattlerRowFrame>; 10],
) -> [Option<VisualBattleBattlerRows>; 2] {
    let mut rows = [None; 2];
    for row in live.iter().flatten() {
        if row.oam.entries.is_empty() {
            continue;
        }
        // The two supported source roots allocate this strip first. Reject
        // partial/multiple strips until an explicit per-piece compositor exists.
        if row.row_count != 2 || row.oam.rows.iter().flatten().any(|visible| !visible) {
            continue;
        }
        let y_start = row
            .oam
            .entries
            .iter()
            .map(|entry| i32::from(entry[0]) - 16)
            .min()
            .unwrap();
        let y_end = row
            .oam
            .entries
            .iter()
            .map(|entry| i32::from(entry[0]) - 8)
            .max()
            .unwrap();
        let effect = &animation.bg_events[row.bg_event_index];
        let redrawn = animation.bg_events.iter().any(|candidate| {
            candidate.effect_id == "BATTLE_BG_EFFECT_SHOW_MON"
                && candidate.frame > effect.frame
                && candidate.frame <= animation.frame
                && target_player_for_effect(animation, candidate) == row.player_side
        });
        rows[usize::from(!row.player_side)] = Some(VisualBattleBattlerRows {
            source_y: Vec2::new(y_start as f32, y_end as f32),
            bg_cleared: animation.frame > row.spawn_frame && !redrawn,
        });
    }
    rows
}

/// Source SetLCDStatCustoms1 + end increment + FillLYOverridesBackup: player
/// [47,95), enemy [0,55). SCX samples right, so a rightward move is negative.
fn immersive_row_prototype_tackle_scx(animation: &VisibleMoveAnimation) -> Option<[i8; 0x5f]> {
    let effect = animation.bg_events.iter().find(|effect| {
        effect.effect_id == "BATTLE_BG_EFFECT_TACKLE" && effect.frame <= animation.frame
    })?;
    let distance = visible_tackle_lunge_offset(animation.frame.saturating_sub(effect.frame))?;
    let player = target_player_for_effect(animation, effect);
    let mut rows = [0; 0x5f];
    let (start, end) = if player { (47, 95) } else { (0, 55) };
    rows[start..end].fill(if player {
        -distance as i8
    } else {
        distance as i8
    });
    Some(rows)
}
