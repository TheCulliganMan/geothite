//! Native Dex/nickname widgets projected from the saved Safari completion owner.

use anyhow::{Context, Result, ensure};
use crystal_core::systems::safari::{SafariPhase, SafariResolution};
use crystal_core::systems::safari_catch_presentation::{
    self as capture, SafariCatchCommand as Command, SafariCatchStage as Stage, SafariNameDraft,
};

use super::{
    BevyRuntimeShell, NameInputCase, PendingNameInput, VisibleNameChoice, VisiblePokedexControls,
    mark_runtime_snapshot_dirty, queue_visible_pokemon_cry, visible_nickname_prompt_pages,
    visible_pokemon_nickname_label,
};

fn saved(shell: &BevyRuntimeShell) -> Result<Option<capture::SafariCatchPresentation>> {
    Ok(capture::presentation(shell.shell.session().state())?)
}

/// Let the existing production widgets handle their own buttons.
pub(super) fn native_surface_owns(shell: &BevyRuntimeShell) -> bool {
    saved(shell)
        .ok()
        .flatten()
        .is_some_and(|saved| match saved.stage {
            Stage::DexEntry => shell.pokedex_menu_open,
            Stage::NicknamePrompt => shell.pending_name_choice.is_some(),
            Stage::Naming => shell.pending_name_input.is_some(),
            _ => false,
        })
}

fn command(shell: &mut BevyRuntimeShell, command: Command) -> Result<()> {
    shell.shell.update_safari_catch_presentation(command)?;
    mark_runtime_snapshot_dirty(shell);
    Ok(())
}

/// Restore the same native widget after advancing a saved stage or Continue.
pub(super) fn restore(shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(saved) = saved(shell)? else {
        return Ok(());
    };
    match saved.stage {
        Stage::CaughtText | Stage::DexNotice | Stage::TransferText => return Ok(()),
        Stage::Naming if shell.pending_name_input.is_some() => {
            let input = shell
                .pending_name_input
                .as_mut()
                .context("native name widget missing")?;
            input.value.clone_from(&saved.name_draft.value);
            input.cursor_column = saved.name_draft.cursor_column;
            input.cursor_row = saved.name_draft.cursor_row;
            input.case = if saved.name_draft.uppercase {
                NameInputCase::Upper
            } else {
                NameInputCase::Lower
            };
            return Ok(());
        }
        Stage::DexEntry if shell.pokedex_menu_open => {
            shell.pokedex_detail_page = saved.dex_page;
            return Ok(());
        }
        _ => {}
    }
    let snapshot = shell.shell.presentation_snapshot()?;
    let active = snapshot
        .safari
        .as_ref()
        .context("capture screen lost its permit")?;
    let SafariPhase::Presenting { result, .. } = &active.phase else {
        anyhow::bail!("capture screen lost its result");
    };
    let SafariResolution::Caught(stored) = &result.resolution else {
        anyhow::bail!("capture screen lost its catch");
    };
    let species_id = &stored.pokemon.species.id;
    let mut opened = false;
    match saved.stage {
        Stage::DexEntry if !shell.pokedex_menu_open => {
            let entry = snapshot
                .presentation
                .pokedex_entries
                .get(species_id)
                .context("caught species has no native Dex entry")?;
            ensure!(
                saved.dex_page < entry.pages.len(),
                "saved Dex page exceeds native entry"
            );
            shell.pokedex_cursor = snapshot
                .pokemon
                .iter()
                .position(|species| &species.species_id == species_id)
                .context("caught species absent from Dex catalog")?;
            shell.pokedex_controls = VisiblePokedexControls {
                mode: shell.pokedex_controls.mode,
                ..Default::default()
            };
            shell.pokedex_menu_open = true;
            shell.pokedex_detail_open = true;
            shell.pokedex_detail_page = saved.dex_page;
            shell.pokedex_scripted_entry = true;
            queue_visible_pokemon_cry(shell, species_id, "safari_new_pokedex_entry")?;
            opened = true;
        }
        Stage::NicknamePrompt => {
            let mut pages = visible_nickname_prompt_pages(shell, &stored.pokemon.nickname, false)?;
            ensure!(
                saved.nickname_page < pages.len(),
                "saved nickname prompt page exceeds native text"
            );
            for _ in 0..saved.nickname_page {
                pages.pop_front();
            }
            if let Some(choice) = shell.pending_name_choice.as_mut() {
                choice.nickname_pages = pages;
                choice.selected = usize::from(!saved.selected_yes);
                return Ok(());
            }
            shell.pending_name_choice = Some(VisibleNameChoice {
                nickname_pages: pages,
                options: vec!["YES".into(), "NO".into()],
                selected: usize::from(!saved.selected_yes),
                player_menu: None,
                player_phase: None,
                motion_step: 0,
                motion_frames_remaining: 0,
                pending_player_name: None,
            });
            opened = true;
        }
        Stage::Naming if shell.pending_name_input.is_none() => {
            shell.pending_name_input = Some(PendingNameInput {
                label: visible_pokemon_nickname_label(&stored.pokemon.nickname),
                value: saved.name_draft.value,
                max_length: 10,
                cursor_column: saved.name_draft.cursor_column,
                cursor_row: saved.name_draft.cursor_row,
                case: if saved.name_draft.uppercase {
                    NameInputCase::Upper
                } else {
                    NameInputCase::Lower
                },
            });
            opened = true;
        }
        _ => {}
    }
    if opened {
        mark_runtime_snapshot_dirty(shell);
    }
    Ok(())
}

pub(super) fn sync_choice(shell: &mut BevyRuntimeShell) -> Result<()> {
    if saved(shell)?.is_some_and(|saved| saved.stage == Stage::NicknamePrompt) {
        let selected = shell
            .pending_name_choice
            .as_ref()
            .context("native nickname choice missing")?
            .selected;
        command(
            shell,
            Command::SelectNickname {
                selected_yes: selected == 0,
            },
        )?;
    }
    Ok(())
}

pub(super) fn advance_prompt_page(shell: &mut BevyRuntimeShell) -> Result<()> {
    if let Some(saved) = saved(shell)?.filter(|saved| saved.stage == Stage::NicknamePrompt) {
        command(
            shell,
            Command::SetNicknamePage {
                page: saved.nickname_page + 1,
            },
        )?;
    }
    Ok(())
}

pub(super) fn confirm_choice(shell: &mut BevyRuntimeShell) -> Result<bool> {
    if !saved(shell)?.is_some_and(|saved| saved.stage == Stage::NicknamePrompt) {
        return Ok(false);
    }
    command(shell, Command::ConfirmNickname)?;
    restore(shell)?;
    Ok(true)
}

pub(super) fn decline_choice(shell: &mut BevyRuntimeShell) -> Result<bool> {
    if !saved(shell)?.is_some_and(|saved| saved.stage == Stage::NicknamePrompt) {
        return Ok(false);
    }
    command(
        shell,
        Command::SelectNickname {
            selected_yes: false,
        },
    )?;
    shell.pending_name_choice = None;
    confirm_choice(shell)
}

pub(super) fn sync_name(shell: &mut BevyRuntimeShell) -> Result<()> {
    if saved(shell)?.is_some_and(|saved| saved.stage == Stage::Naming) {
        let input = shell
            .pending_name_input
            .as_ref()
            .context("native Safari naming input missing")?;
        let draft = SafariNameDraft {
            value: input.value.clone(),
            cursor_column: input.cursor_column,
            cursor_row: input.cursor_row,
            uppercase: input.case == NameInputCase::Upper,
        };
        command(shell, Command::UpdateNameDraft { draft })?;
    }
    Ok(())
}

pub(super) fn finish_name(shell: &mut BevyRuntimeShell, input: &PendingNameInput) -> Result<bool> {
    if !saved(shell)?.is_some_and(|saved| saved.stage == Stage::Naming) {
        return Ok(false);
    }
    command(
        shell,
        Command::UpdateNameDraft {
            draft: SafariNameDraft {
                value: input.value.clone(),
                cursor_column: input.cursor_column,
                cursor_row: input.cursor_row,
                uppercase: input.case == NameInputCase::Upper,
            },
        },
    )?;
    command(shell, Command::FinishName)?;
    shell.pending_name_input = None;
    shell.pending_name_choice = None;
    Ok(true)
}

pub(super) fn sync_dex_page(shell: &mut BevyRuntimeShell) -> Result<()> {
    if saved(shell)?.is_some_and(|saved| saved.stage == Stage::DexEntry) {
        command(
            shell,
            Command::SetDexPage {
                page: shell.pokedex_detail_page,
            },
        )?;
    }
    Ok(())
}

pub(super) fn finish_dex(shell: &mut BevyRuntimeShell) -> Result<bool> {
    if !saved(shell)?.is_some_and(|saved| saved.stage == Stage::DexEntry) {
        return Ok(false);
    }
    command(shell, Command::Advance)?;
    restore(shell)?;
    Ok(true)
}
