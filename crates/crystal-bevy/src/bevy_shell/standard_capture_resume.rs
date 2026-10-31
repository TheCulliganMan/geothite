//! Keep an already resolved wild catch across an interrupted presentation.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use super::{
    BevyRuntimeShell, NameInputCase, PendingNameInput, PendingStandardCapture,
    continue_visible_capture_after_owned_surface, mark_runtime_snapshot_dirty,
    visible_name_input_grid_height, visible_name_input_grid_width, visible_pokemon_nickname_label,
    visible_static_wild_source,
};

pub(super) const KEY: &str = "GEOTHITE_STANDARD_CAPTURE_V1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedCapture {
    version: u8,
    species: String,
    level: u8,
    outcome: crystal_core::battle::capture::CaptureOutcome,
    origin: Option<crystal_runtime::RuntimeStaticWildBattleOrigin>,
    default_name: String,
    prompt_for_nickname: bool,
    choice: Option<(usize, usize)>,
    name: Option<SavedName>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedName {
    value: String,
    column: usize,
    row: usize,
    lower_case: bool,
}

/// Encode presentation state only; the ball and battle already live in core.
pub(super) fn encode(shell: &BevyRuntimeShell) -> Result<Option<String>> {
    let Some(pending) = shell.pending_standard_capture.as_ref() else {
        return Ok(None);
    };
    let snapshot = shell.shell.presentation_snapshot()?;
    let battle = snapshot
        .battle
        .as_ref()
        .context("pending catch without an active wild battle")?;
    // Contest/tutorial presentation has its own completion rules.
    if matches!(
        battle.battle_type.as_str(),
        "BATTLETYPE_CONTEST" | "BATTLETYPE_TUTORIAL"
    ) {
        return Ok(None);
    }
    ensure!(
        pending.outcome.caught && !pending.outcome.blocked && !pending.outcome.storage_full,
        "only a successful unresolved catch can be checkpointed"
    );
    let saved = SavedCapture {
        version: 1,
        species: battle.enemy_pokemon.species.id.clone(),
        level: battle.enemy_pokemon.level,
        outcome: pending.outcome.clone(),
        origin: pending.scripted_static_wild.clone(),
        default_name: pending.default_name.clone(),
        prompt_for_nickname: pending.prompt_for_nickname,
        choice: shell
            .pending_name_choice
            .as_ref()
            .map(|choice| (choice.nickname_pages.len(), choice.selected)),
        name: shell.pending_name_input.as_ref().map(|name| SavedName {
            value: name.value.clone(),
            column: name.cursor_column,
            row: name.cursor_row,
            lower_case: matches!(name.case, NameInputCase::Lower),
        }),
    };
    Ok(Some(
        serde_json::to_string(&saved).context("encode pending standard catch")?,
    ))
}

/// Resume the naming step with the same resolved catch and paid ball.
/// Core capture completion still performs all normal party/PC/Dex validation.
pub(super) fn restore(shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(encoded) = shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .get(KEY)
        .cloned()
    else {
        return Ok(());
    };
    let saved: SavedCapture =
        serde_json::from_str(&encoded).context("decode pending standard catch")?;
    let snapshot = shell.shell.presentation_snapshot()?;
    let battle = snapshot
        .battle
        .as_ref()
        .context("saved catch has no active wild battle")?;
    ensure!(
        saved.version == 1
            && saved.outcome.caught
            && !saved.outcome.blocked
            && !saved.outcome.storage_full,
        "invalid saved standard catch outcome"
    );
    ensure!(
        matches!(
            battle.kind,
            crystal_runtime::RuntimeBattleKind::Wild { .. }
                | crystal_runtime::RuntimeBattleKind::StaticWild { .. }
        ),
        "saved catch cannot capture a trainer's Pokemon"
    );
    ensure!(
        saved.species == battle.enemy_pokemon.species.id
            && saved.level == battle.enemy_pokemon.level
            && saved.origin == visible_static_wild_source(&snapshot, battle),
        "saved catch does not match its active encounter"
    );
    ensure!(
        saved.prompt_for_nickname
            && saved.default_name == super::canonical_species_display_name(&saved.species),
        "saved standard catch must retain its native naming rules"
    );
    let ball = saved
        .outcome
        .ball_id
        .as_ref()
        .context("saved catch has no paid ball identity")?;
    ensure!(
        shell
            .shell
            .runtime()
            .data()
            .items
            .get(ball)
            .is_some_and(|item| item.battle_capture_ball && item.battle_usable),
        "saved catch references an invalid capture ball"
    );
    if let Some(name) = &saved.name {
        ensure!(
            name.value.chars().count() <= 10
                && !name.value.chars().any(char::is_control)
                && name.column < visible_name_input_grid_width()
                && name.row < visible_name_input_grid_height(),
            "invalid saved caught-Pokemon name input"
        );
    }
    if let Some((pages, selected)) = saved.choice {
        ensure!(pages > 0 && selected < 2, "invalid saved capture choice");
    }
    shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .memory
        .remove(KEY);
    shell.pending_standard_capture = Some(PendingStandardCapture {
        outcome: saved.outcome,
        scripted_static_wild: saved.origin,
        default_name: saved.default_name,
        prompt_for_nickname: saved.prompt_for_nickname,
    });
    // Do not resolve another throw or consume another ball. Continue returns
    // directly to naming the confirmed caught Pokemon, preserving typed input.
    continue_visible_capture_after_owned_surface(shell)?;
    if let Some(name) = saved.name {
        let default = &shell
            .pending_standard_capture
            .as_ref()
            .context("restored catch owner")?
            .default_name;
        shell.pending_name_choice = None;
        shell.pending_name_input = Some(PendingNameInput {
            label: visible_pokemon_nickname_label(default),
            value: name.value,
            max_length: 10,
            cursor_column: name.column,
            cursor_row: name.row,
            case: if name.lower_case {
                NameInputCase::Lower
            } else {
                NameInputCase::Upper
            },
        });
    } else if let Some((pages, selected)) = saved.choice {
        let choice = shell
            .pending_name_choice
            .as_mut()
            .context("restored capture choice")?;
        ensure!(
            pages <= choice.nickname_pages.len(),
            "saved capture page is outside its native prompt"
        );
        while choice.nickname_pages.len() > pages {
            choice.nickname_pages.pop_front();
        }
        choice.selected = selected;
    }
    mark_runtime_snapshot_dirty(shell);
    Ok(())
}
